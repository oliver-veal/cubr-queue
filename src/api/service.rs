use crate::domain::{
    entity::Render,
    load_balance,
    repository::{JobRepository, RenderRepository},
};
use anyhow::{anyhow, Result};
use libcubr::{event::event::*, rpc::rpc::ServiceResponse, service::queue::*};
use tracing::info;

#[derive(Clone, Debug)]
pub struct QueueServiceImpl<RR, JR, E>
where
    RR: RenderRepository,
    JR: JobRepository,
    E: EventTransport,
{
    render: RR,
    job: JR,
    event: E,
}

impl<RR, JR, E> QueueServiceImpl<RR, JR, E>
where
    RR: RenderRepository,
    JR: JobRepository,
    E: EventTransport,
{
    pub fn new(render: RR, job: JR, event: E) -> Self {
        Self { render, job, event }
    }
}

#[async_trait::async_trait]
impl<RR, JR, E> QueueServiceRPC for QueueServiceImpl<RR, JR, E>
where
    RR: RenderRepository,
    JR: JobRepository,
    E: EventTransport,
{
    async fn pop(&self, req: PopRequest) -> Result<ServiceResponse<PopResponse, PopError>> {
        info!("Pop request: {:?}", req);

        let queue = self.render.load_queue().await?;

        let mut render = match load_balance::select_render(queue).await {
            Some(render) => render,
            None => return Ok(ServiceResponse::Err(PopError::QueueEmpty)),
        };

        if render.is_queue_drained() {
            return Ok(ServiceResponse::Err(PopError::QueueEmpty));
        }

        let job = match render.get_job(req.worker_id.clone()) {
            Some(job) => job,
            None => {
                // This branch shouldn't happen.
                self.render.delete(&render.id).await?;
                return Err(anyhow!("Job pop out of range: {:?}", render));
            }
        };

        if render.is_first() {
            self.event
                .publish(&Event::new(Payload::RenderRunning(RenderRunning {
                    id: render.id.clone(),
                })))
                .await?;
        }

        render.advance_pointer();

        self.render.update_pointer(&render).await?;

        self.job.store(&job).await?;

        self.event
            .publish(&Event::new(Payload::JobRunning(JobRunning {
                user_id: job.user_id.clone(),
                frame: job.frame,
                slice: job.slice,
                render_id: render.id,
                worker_id: req.worker_id,
            })))
            .await?;

        let resp = PopResponse {
            user_id: job.user_id,
            render_id: job.render_id,
            frame: job.frame,
            slice: job.slice,
            file_id: job.file_id,
            file_version: job.file_version,
            total_slices: job.total_slices,
            worker_id: job.worker_id,
            subscription_item_id: render.subscription_item_id,
        };

        info!("Pop response: {:?}", resp);
        Ok(ServiceResponse::Ok(resp))
    }

    async fn get_scale_target(&self) -> Result<GetScaleTargetResponse> {
        let renders = self.render.load_queue().await?;

        let mut target: usize = 0;

        for r in renders {
            let total_jobs: usize = r.total_jobs.try_into()?;
            let completed_jobs: usize = r.completed_jobs.try_into()?;
            let remaining_jobs = total_jobs - completed_jobs;
            info!(
                r.id,
                total_jobs, completed_jobs, remaining_jobs, "Remaining jobs"
            );
            target += remaining_jobs;
        }

        info!("GetScaleTarget response: {:?}", target);

        Ok(GetScaleTargetResponse { target })
    }
}

#[async_trait::async_trait]
impl<RR, JR, E> QueueServiceEvents for QueueServiceImpl<RR, JR, E>
where
    RR: RenderRepository,
    JR: JobRepository,
    E: EventTransport,
{
    async fn render_cancel_requested(&self, _: Header, event: RenderCancelRequested) -> Result<()> {
        info!("Render canceled: {:?}", event);

        self.render.delete(&event.id).await?;

        self.event
            .publish(&Event::new(Payload::RenderCanceled(RenderCanceled {
                id: event.id,
            })))
            .await?;

        Ok(())
    }

    async fn render_submitted(&self, _: Header, event: RenderSubmitted) -> Result<()> {
        info!("Render submitted: {:?}", event);

        let render = Render::new(
            event.user_id,
            event.id.clone(),
            event.file_id,
            event.file_version,
            event.frame_start,
            event.frame_end,
            event.step,
            event.slices,
            event.subscription_item_id,
        );

        self.render.store(&render).await?;

        self.event
            .publish(&Event::new(Payload::RenderPending(RenderPending {
                id: event.id,
            })))
            .await?;

        Ok(())
    }

    async fn job_canceled(&self, _: Header, event: JobCanceled) -> Result<()> {
        info!("Job canceled: {:?}", event);

        self.job
            .delete(event.render_id, event.frame, event.slice)
            .await?;

        Ok(())
    }

    async fn job_complete(&self, _: Header, event: JobComplete) -> Result<()> {
        info!("Job complete: {:?}", event);

        self.job
            .delete(event.render_id.clone(), event.frame, event.slice)
            .await?;

        let render = match self
            .render
            .increment_completed_jobs(&event.render_id)
            .await?
        {
            Some(render) => render,
            // This is the case where a render was canceled but a job kept going.
            None => return Ok(()),
        };

        // Check if the render is complete.

        let inprogress_jobs = self.job.count(event.render_id.clone()).await?;

        if inprogress_jobs == 0 && render.is_complete() {
            self.render.delete(&event.render_id).await?;

            self.event
                .publish(&Event::new(Payload::RenderComplete(RenderComplete {
                    id: event.render_id,
                })))
                .await?;
        }

        Ok(())
    }

    async fn job_failed(&self, _: Header, event: JobFailed) -> Result<()> {
        info!("Job failed: {:?}", event);

        self.job
            .delete(event.render_id.clone(), event.frame, event.slice)
            .await?;

        let render = match self
            .render
            .increment_completed_jobs(&event.render_id)
            .await?
        {
            Some(render) => render,
            // This is the case where a render was canceled or failed but a job kept going.
            None => return Ok(()),
        };

        // Check if the render is complete.

        // If the render is a still (i.e. frame_start == frame_end) then emit RenderFailed.

        if render.frame_start == render.frame_end {
            self.render.delete(&event.render_id).await?;

            self.event
                .publish(&Event::new(Payload::RenderFailed(RenderFailed {
                    id: event.render_id,
                })))
                .await?;

            return Ok(());
        }

        let inprogress_jobs = self.job.count(event.render_id.clone()).await?;

        if inprogress_jobs == 0 && render.is_complete() {
            self.render.delete(&event.render_id).await?;

            self.event
                .publish(&Event::new(Payload::RenderComplete(RenderComplete {
                    id: event.render_id,
                })))
                .await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<RR, JR, E> EventRouter for QueueServiceImpl<RR, JR, E>
where
    RR: RenderRepository,
    JR: JobRepository,
    E: EventTransport,
{
    async fn route(&self, event: &Event) -> Result<()> {
        let header = event.header.clone();
        match event.payload.clone() {
            Payload::RenderSubmitted(e) => self.render_submitted(header, e).await,
            Payload::RenderCancelRequested(e) => self.render_cancel_requested(header, e).await,
            Payload::JobComplete(e) => self.job_complete(header, e).await,
            Payload::JobFailed(e) => self.job_failed(header, e).await,
            Payload::JobCanceled(e) => self.job_canceled(header, e).await,
            _ => Ok(()),
        }
    }
}

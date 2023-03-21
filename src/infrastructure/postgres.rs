use anyhow::{Context, Result};
use sqlx::{postgres::PgRow, types::Uuid, FromRow, PgPool, Row};

use crate::domain::{
    entity::{Job, Render},
    repository::{JobRepository, RenderRepository},
};

#[derive(Clone, Debug)]
pub struct PgRenderRepository {
    pool: PgPool,
}

impl PgRenderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RenderRepository for PgRenderRepository {
    async fn load_queue(&self) -> Result<Vec<Render>> {
        let renders: Vec<Render> = sqlx::query_as("SELECT * FROM queue.queue")
            .fetch_all(&self.pool)
            .await
            .context("RenderRepository::load_queue")?;

        Ok(renders)
    }

    async fn load(&self, id: &str) -> Result<Option<Render>> {
        let render: Option<Render> = sqlx::query_as("SELECT * FROM queue.queue WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("RenderRepository::load")?;

        Ok(render)
    }

    async fn store(&self, render: &Render) -> Result<()> {
        let user_id_uuid: Uuid = render.user_id.parse()?;
        let file_id_uuid: Uuid = render.file_id.parse()?;

        sqlx::query(
            r#"
            INSERT INTO queue.queue (id, user_id, file_id, file_version, frame_start, frame_end, step, slices, pointer_frame, pointer_slice, total_jobs, completed_jobs, subscription_item_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (user_id, id) DO UPDATE SET
                user_id = $2,
                file_id = $3,
                file_version = $4,
                frame_start = $5,
                frame_end = $6,
                step = $7,
                slices = $8,
                pointer_frame = $9,
                pointer_slice = $10,
                total_jobs = $11,
                completed_jobs = $12,
                subscription_item_id = $13
            "#,
        )
        .bind(&render.id)
        .bind(&user_id_uuid)
        .bind(&file_id_uuid)
        .bind(&render.file_version)
        .bind(&render.frame_start)
        .bind(&render.frame_end)
        .bind(&render.step)
        .bind(&render.slices)
        .bind(&render.pointer_frame)
        .bind(&render.pointer_slice)
        .bind(&render.total_jobs)
        .bind(&render.completed_jobs)
        .bind(&render.subscription_item_id)
        .execute(&self.pool)
        .await
        .context("RenderRepository::store")?;

        Ok(())
    }

    async fn update_pointer(&self, render: &Render) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE queue.queue
            SET pointer_frame = $1, pointer_slice = $2
            WHERE id = $3
            "#,
        )
        .bind(&render.pointer_frame)
        .bind(&render.pointer_slice)
        .bind(&render.id)
        .execute(&self.pool)
        .await
        .context("RenderRepository::update_pointer")?;

        Ok(())
    }

    async fn increment_completed_jobs(&self, id: &str) -> Result<Option<Render>> {
        let render: Option<Render> = sqlx::query_as(
            r#"
            UPDATE queue.queue
            SET completed_jobs = completed_jobs + 1
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("RenderRepository::increment_completed_jobs")?;

        Ok(render)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM queue.queue WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("RenderRepository::delete")?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Render {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id: String = row.try_get("id")?;
        let user_id: Uuid = row.try_get("user_id")?;
        let file_id: Uuid = row.try_get("file_id")?;
        let file_version: i32 = row.try_get("file_version")?;
        let frame_start: i32 = row.try_get("frame_start")?;
        let frame_end: i32 = row.try_get("frame_end")?;
        let step: i32 = row.try_get("step")?;
        let slices: i32 = row.try_get("slices")?;
        let pointer_frame: i32 = row.try_get("pointer_frame")?;
        let pointer_slice: i32 = row.try_get("pointer_slice")?;
        let total_jobs: i32 = row.try_get("total_jobs")?;
        let completed_jobs: i32 = row.try_get("completed_jobs")?;
        let subscription_item_id: String = row.try_get("subscription_item_id")?;

        Ok(Self {
            id: id,
            user_id: user_id.to_string(),
            file_id: file_id.to_string(),
            file_version,
            frame_start,
            frame_end,
            step,
            slices,
            pointer_frame,
            pointer_slice,
            total_jobs,
            completed_jobs,
            subscription_item_id,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PgJobRepository {
    pool: PgPool,
}

impl PgJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for PgJobRepository {
    async fn store(&self, job: &Job) -> Result<()> {
        let user_id_uuid: Uuid = job.user_id.parse()?;

        sqlx::query(
            r#"
            INSERT INTO queue.jobs (user_id, render_id, frame, slice, worker_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&user_id_uuid)
        .bind(&job.render_id)
        .bind(&job.frame)
        .bind(&job.slice)
        .bind(&job.worker_id)
        .execute(&self.pool)
        .await
        .context("JobRepository::store")?;

        Ok(())
    }

    async fn delete(&self, render_id: String, frame: i32, slice: i32) -> Result<()> {
        sqlx::query("DELETE FROM queue.jobs WHERE render_id = $1 AND frame = $2 AND slice = $3")
            .bind(&render_id)
            .bind(&frame)
            .bind(&slice)
            .execute(&self.pool)
            .await
            .context("JobRepository::delete")?;

        Ok(())
    }

    async fn count(&self, render_id: String) -> Result<i64> {
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM queue.jobs WHERE render_id = $1")
            .bind(&render_id)
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(&self.pool)
            .await
            .context("JobRepository::count")?;

        Ok(count)
    }
}

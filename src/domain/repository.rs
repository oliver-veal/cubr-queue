use anyhow::Result;

use super::entity::{Job, Render};

#[async_trait::async_trait]
pub trait RenderRepository: Clone + Send + Sync {
    async fn load_queue(&self) -> Result<Vec<Render>>;

    async fn load(&self, id: &str) -> Result<Option<Render>>;

    async fn store(&self, render: &Render) -> Result<()>;

    async fn update_pointer(&self, render: &Render) -> Result<()>;

    async fn increment_completed_jobs(&self, id: &str) -> Result<Option<Render>>;

    async fn delete(&self, id: &str) -> Result<()>;
}

#[async_trait::async_trait]
pub trait JobRepository: Clone + Send + Sync {
    async fn store(&self, job: &Job) -> Result<()>;

    async fn delete(&self, render_id: String, frame: i32, slice: i32) -> Result<()>;

    async fn count(&self, render_id: String) -> Result<i64>;
}

use async_trait::async_trait;
use shared::{DomainResult, JobStatus};
use uuid::Uuid;

use crate::domain::entities::ScheduleJob;

#[async_trait]
pub trait ScheduleJobRepository: Send + Sync {
    /// Create a new schedule job
    async fn create(&self, job: ScheduleJob) -> DomainResult<ScheduleJob>;

    /// Find job by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScheduleJob>>;

    /// Update job status
    async fn update_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error_message: Option<String>,
    ) -> DomainResult<()>;

    /// Mark job as completed
    async fn mark_completed(&self, id: Uuid) -> DomainResult<()>;

    /// Mark job as failed
    async fn mark_failed(&self, id: Uuid, error_message: String) -> DomainResult<()>;
}

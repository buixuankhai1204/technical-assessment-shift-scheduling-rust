use async_trait::async_trait;
use shared::DomainResult;
use uuid::Uuid;

use crate::domain::entities::ShiftAssignment;

/// Repository trait for ShiftAssignment operations
#[async_trait]
pub trait ShiftAssignmentRepository: Send + Sync {
    /// Create shift assignments in batch
    async fn create_batch(&self, assignments: Vec<ShiftAssignment>) -> DomainResult<()>;

    /// Find all assignments for a schedule job
    async fn find_by_job_id(&self, job_id: Uuid) -> DomainResult<Vec<ShiftAssignment>>;

    /// Delete all assignments for a job
    async fn delete_by_job_id(&self, job_id: Uuid) -> DomainResult<()>;
}

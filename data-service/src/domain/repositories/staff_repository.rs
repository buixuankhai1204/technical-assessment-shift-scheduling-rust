use async_trait::async_trait;
use shared::{DomainResult, PaginationParams, StaffStatus};
use uuid::Uuid;

use crate::api::requests::{CreateStaffRequest, UpdateStaffRequest};
use crate::domain::entities::Staff;

#[async_trait]
pub trait StaffRepository: Send + Sync {
    /// Create a new staff member
    async fn create(&self, request: CreateStaffRequest) -> DomainResult<Staff>;

    /// Find staff by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Staff>>;

    /// Find staff by email
    #[allow(dead_code)]
    async fn find_by_email(&self, email: &str) -> DomainResult<Option<Staff>>;

    /// List all staff with pagination
    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<Staff>, u64)>;

    /// List staff by status
    #[allow(dead_code)]
    async fn list_by_status(
        &self,
        status: StaffStatus,
        params: PaginationParams,
    ) -> DomainResult<(Vec<Staff>, u64)>;

    /// Update staff by ID
    async fn update(&self, id: Uuid, request: UpdateStaffRequest) -> DomainResult<Staff>;

    /// Delete staff by ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Get staff by group ID
    async fn find_by_group_id(&self, group_id: Uuid) -> DomainResult<Vec<Staff>>;
}

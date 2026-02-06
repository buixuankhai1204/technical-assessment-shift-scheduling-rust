use async_trait::async_trait;
use shared::{DomainResult, PaginationParams};
use uuid::Uuid;

use crate::api::requests::{CreateGroupRequest, UpdateGroupRequest};
use crate::domain::entities::StaffGroup;

/// Repository trait for StaffGroup operations
#[async_trait]
pub trait GroupRepository: Send + Sync {
    /// Create a new staff group
    async fn create(&self, request: CreateGroupRequest) -> DomainResult<StaffGroup>;

    /// Find group by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<StaffGroup>>;

    /// List all groups with pagination
    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<StaffGroup>, u64)>;

    /// List child groups by parent ID
    #[allow(dead_code)]
    async fn list_by_parent_id(&self, parent_id: Uuid) -> DomainResult<Vec<StaffGroup>>;

    /// Update group by ID
    async fn update(&self, id: Uuid, request: UpdateGroupRequest) -> DomainResult<StaffGroup>;

    /// Delete group by ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Get all descendant group IDs (recursive)
    async fn get_descendant_ids(&self, group_id: Uuid) -> DomainResult<Vec<Uuid>>;
}

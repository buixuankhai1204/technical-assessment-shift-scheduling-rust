use async_trait::async_trait;
use shared::{DomainResult, PaginationParams};
use uuid::Uuid;

use crate::api::requests::{CreateGroupRequest, UpdateGroupRequest};
use crate::domain::entities::{GroupWithMembers, StaffGroup};

#[async_trait]
pub trait GroupRepository: Send + Sync {
    /// Create a new staff group
    async fn create(&self, request: CreateGroupRequest) -> DomainResult<StaffGroup>;

    /// Find group by ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<StaffGroup>>;

    /// List all groups with pagination
    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<StaffGroup>, u64)>;

    /// Update group by ID
    async fn update(&self, id: Uuid, request: UpdateGroupRequest) -> DomainResult<StaffGroup>;

    /// Delete group by ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Find group by name
    async fn find_by_name(&self, name: &str) -> DomainResult<Option<StaffGroup>>;

    /// Get all members in a group hierarchy, grouped by subgroup.
    /// Returns (groups_with_members, total_unique_active_members).
    async fn get_resolved_members(
        &self,
        group_id: Uuid,
    ) -> DomainResult<(Vec<GroupWithMembers>, u64)>;
}

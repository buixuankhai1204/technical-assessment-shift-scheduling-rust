use async_trait::async_trait;
use shared::DomainResult;
use uuid::Uuid;

use crate::domain::entities::GroupMembership;

/// Repository trait for GroupMembership operations
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Add staff to group
    async fn add_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<GroupMembership>;

    /// Remove staff from group
    async fn remove_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<()>;

    /// Get all memberships for a staff member
    #[allow(dead_code)]
    async fn find_by_staff_id(&self, staff_id: Uuid) -> DomainResult<Vec<GroupMembership>>;

    /// Get all memberships for a group
    async fn find_by_group_id(&self, group_id: Uuid) -> DomainResult<Vec<GroupMembership>>;

    /// Check if staff is member of group
    #[allow(dead_code)]
    async fn is_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<bool>;
}

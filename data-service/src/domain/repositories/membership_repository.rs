use async_trait::async_trait;
use shared::DomainResult;
use uuid::Uuid;

use crate::domain::entities::GroupMembership;

#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Add staff to group
    async fn add_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<GroupMembership>;

    /// Remove staff from group
    async fn remove_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<()>;
}

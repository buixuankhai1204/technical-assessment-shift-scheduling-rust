use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::GroupMembership;

/// Request to add staff to group
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub staff_id: Uuid,
}

/// Request to remove staff from group
#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveMemberRequest {
    pub staff_id: Uuid,
}

/// Membership response DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MembershipResponse {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub group_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<GroupMembership> for MembershipResponse {
    fn from(membership: GroupMembership) -> Self {
        Self {
            id: membership.id,
            staff_id: membership.staff_id,
            group_id: membership.group_id,
            created_at: membership.created_at,
        }
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Group membership entity (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct GroupMembership {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub group_id: Uuid,
    pub created_at: DateTime<Utc>,
}

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

/// Batch import request for staff
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportStaffRequest {
    pub staff: Vec<crate::domain::entities::staff::CreateStaffRequest>,
}

/// Batch import request for groups
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportGroupsRequest {
    pub groups: Vec<crate::domain::entities::group::CreateGroupRequest>,
}

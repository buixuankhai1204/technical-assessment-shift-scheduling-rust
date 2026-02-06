use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::{GroupMembership, Staff, StaffGroup};

/// Membership serializer DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MembershipSerializer {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub group_id: Uuid,
    pub staff_name: String,
    pub staff_email: String,
    pub group_name: String,
    pub created_at: DateTime<Utc>,
}

impl MembershipSerializer {
    pub fn new(membership: GroupMembership, staff: &Staff, group: &StaffGroup) -> Self {
        Self {
            id: membership.id,
            staff_id: membership.staff_id,
            group_id: membership.group_id,
            staff_name: staff.name.clone(),
            staff_email: staff.email.clone(),
            group_name: group.name.clone(),
            created_at: membership.created_at,
        }
    }
}

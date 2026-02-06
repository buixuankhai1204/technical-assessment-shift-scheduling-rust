use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to add staff to group
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub staff_id: Uuid,
}

/// Request to remove staff from group
#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveMemberRequest {
    pub staff_id: Uuid,
}

use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to add staff to group
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub staff_id: Uuid,
}

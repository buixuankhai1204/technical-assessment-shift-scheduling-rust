use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to create a new staff group
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// Request to update a staff group
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub parent_id: Option<Uuid>,
}

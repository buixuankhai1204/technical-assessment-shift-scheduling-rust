use serde::Deserialize;
use shared::StaffStatus;
use utoipa::ToSchema;

/// Request to create a new staff member
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStaffRequest {
    pub name: String,
    pub email: String,
    pub position: String,
    #[serde(default)]
    pub status: Option<StaffStatus>,
}

/// Request to update a staff member
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStaffRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub position: Option<String>,
    pub status: Option<StaffStatus>,
}

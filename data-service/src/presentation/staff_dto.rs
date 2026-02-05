use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::StaffStatus;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::Staff;

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

/// Staff response DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StaffResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: StaffStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Staff> for StaffResponse {
    fn from(staff: Staff) -> Self {
        Self {
            id: staff.id,
            name: staff.name,
            email: staff.email,
            position: staff.position,
            status: staff.status,
            created_at: staff.created_at,
            updated_at: staff.updated_at,
        }
    }
}

/// Batch import request for staff
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportStaffRequest {
    pub staff: Vec<CreateStaffRequest>,
}

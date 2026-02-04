use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, StaffStatus, Timestamped};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Staff entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Staff {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: StaffStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Identifiable for Staff {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Timestamped for Staff {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

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

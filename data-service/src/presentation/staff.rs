use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::StaffStatus;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::Staff;

/// Staff serializer DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StaffSerializer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: StaffStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Staff> for StaffSerializer {
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, StaffStatus, Timestamped};
use sqlx::FromRow;
use uuid::Uuid;

/// Staff entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

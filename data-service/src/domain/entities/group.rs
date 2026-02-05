use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, Timestamped};
use sqlx::FromRow;
use uuid::Uuid;

/// Staff group entity with hierarchical support
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StaffGroup {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Identifiable for StaffGroup {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Timestamped for StaffGroup {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, Timestamped};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

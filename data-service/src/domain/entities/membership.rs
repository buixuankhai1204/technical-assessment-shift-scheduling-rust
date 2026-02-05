use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Group membership entity (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMembership {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub group_id: Uuid,
    pub created_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::StaffGroup;

/// Staff group serializer DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupSerializer {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<StaffGroup> for GroupSerializer {
    fn from(group: StaffGroup) -> Self {
        Self {
            id: group.id,
            name: group.name,
            parent_id: group.parent_id,
            created_at: group.created_at,
            updated_at: group.updated_at,
        }
    }
}

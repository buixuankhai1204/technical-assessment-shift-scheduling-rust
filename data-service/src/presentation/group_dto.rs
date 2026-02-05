use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::StaffGroup;

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

/// Staff group response DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<StaffGroup> for GroupResponse {
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

/// Batch import request for groups
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportGroupsRequest {
    pub groups: Vec<CreateGroupRequest>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::{GroupWithMembers, StaffGroup};
use crate::presentation::StaffSerializer;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupSerializer {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub parent_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GroupSerializer {
    pub fn new(group: StaffGroup, parent_name: Option<String>) -> Self {
        Self {
            id: group.id,
            name: group.name,
            parent_id: group.parent_id,
            parent_name,
            created_at: group.created_at,
            updated_at: group.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResolvedGroupSerializer {
    pub group_id: Uuid,
    pub group_name: String,
    pub members: Vec<StaffSerializer>,
}

impl From<GroupWithMembers> for ResolvedGroupSerializer {
    fn from(gwm: GroupWithMembers) -> Self {
        Self {
            group_id: gwm.group.id,
            group_name: gwm.group.name,
            members: gwm.members.into_iter().map(StaffSerializer::from).collect(),
        }
    }
}

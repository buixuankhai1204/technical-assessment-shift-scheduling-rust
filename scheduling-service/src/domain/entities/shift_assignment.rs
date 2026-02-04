use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::ShiftType;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Shift assignment entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ShiftAssignment {
    pub id: Uuid,
    pub schedule_job_id: Uuid,
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
    pub created_at: DateTime<Utc>,
}

/// Shift assignment response (for API)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftAssignmentResponse {
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
}

impl From<ShiftAssignment> for ShiftAssignmentResponse {
    fn from(assignment: ShiftAssignment) -> Self {
        Self {
            staff_id: assignment.staff_id,
            date: assignment.date,
            shift: assignment.shift,
        }
    }
}

/// Complete schedule result response
#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleResultResponse {
    pub schedule_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub staff_group_id: Uuid,
    pub assignments: Vec<ShiftAssignmentResponse>,
}

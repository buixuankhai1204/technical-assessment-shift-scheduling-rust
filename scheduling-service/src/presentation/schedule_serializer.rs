use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::JobStatus;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::{ScheduleJob, ShiftAssignment};

#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleJobSerializer {
    pub schedule_id: Uuid,
    pub status: JobStatus,
}

impl From<ScheduleJob> for ScheduleJobSerializer {
    fn from(job: ScheduleJob) -> Self {
        Self {
            schedule_id: job.id,
            status: job.status,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleStatusSerializer {
    pub schedule_id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<ScheduleJob> for ScheduleStatusSerializer {
    fn from(job: ScheduleJob) -> Self {
        Self {
            schedule_id: job.id,
            staff_group_id: job.staff_group_id,
            period_begin_date: job.period_begin_date,
            status: job.status,
            error_message: job.error_message,
            created_at: job.created_at,
            updated_at: job.updated_at,
            completed_at: job.completed_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftAssignmentSerializer {
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: shared::ShiftType,
}

impl From<ShiftAssignment> for ShiftAssignmentSerializer {
    fn from(assignment: ShiftAssignment) -> Self {
        Self {
            staff_id: assignment.staff_id,
            date: assignment.date,
            shift: assignment.shift,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ScheduleResultSerializer {
    pub schedule_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub staff_group_id: Uuid,
    pub assignments: Vec<ShiftAssignmentSerializer>,
}

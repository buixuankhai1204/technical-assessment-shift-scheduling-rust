use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, JobStatus, Timestamped};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Schedule job entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ScheduleJob {
    pub id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Identifiable for ScheduleJob {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl Timestamped for ScheduleJob {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

/// Request to create a new schedule job
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

/// Schedule job response (after submission)
#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleJobResponse {
    pub schedule_id: Uuid,
    pub status: JobStatus,
}

/// Schedule status response
#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleStatusResponse {
    pub schedule_id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<ScheduleJob> for ScheduleStatusResponse {
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

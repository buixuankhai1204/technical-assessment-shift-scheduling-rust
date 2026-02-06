use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::{Identifiable, JobStatus, Timestamped};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::ShiftType;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShiftAssignment {
    pub id: Uuid,
    pub schedule_job_id: Uuid,
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
    pub created_at: DateTime<Utc>,
}

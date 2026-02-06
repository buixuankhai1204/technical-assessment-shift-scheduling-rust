use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "staff_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StaffStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "shift_type", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShiftType {
    Morning,
    Evening,
    DayOff,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

pub trait Timestamped {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

pub trait Identifiable {
    fn id(&self) -> Uuid;
}

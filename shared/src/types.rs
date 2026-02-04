use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Staff status enum
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "staff_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StaffStatus {
    Active,
    Inactive,
}

/// Shift type enum
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "shift_type", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShiftType {
    Morning,
    Evening,
    DayOff,
}

/// Job status enum
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Timestamp trait for entities with created_at and updated_at
pub trait Timestamped {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Identifiable trait for entities with ID
pub trait Identifiable {
    fn id(&self) -> Uuid;
}

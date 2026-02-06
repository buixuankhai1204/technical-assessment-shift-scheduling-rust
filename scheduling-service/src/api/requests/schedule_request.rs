use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to create a new schedule job
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

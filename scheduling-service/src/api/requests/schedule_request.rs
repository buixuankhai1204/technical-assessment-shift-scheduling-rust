use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

#[derive(Debug)]
pub struct ScheduleJobRequest {
    pub job_id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}
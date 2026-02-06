use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Utc};
use shared::JobStatus;
use uuid::Uuid;

use crate::api::requests::CreateScheduleRequest;
use crate::api::state::AppState;
use crate::domain::entities::ScheduleJob;
use crate::infrastructure::ScheduleJobRequest;
use crate::presentation::{
    ScheduleJobSerialize, ScheduleResultSerialize, ScheduleStatusSerialize,
    ShiftAssignmentSerialize,
};

/// Submit a new schedule job
#[utoipa::path(
    post,
    path = "/api/v1/schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 202, description = "Schedule job accepted for processing", body = ScheduleJobSerialize),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "schedules"
)]
pub async fn submit_schedule(
    State(state): State<AppState>,
    Json(request): Json<CreateScheduleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate that the date is a Monday
    if request.period_begin_date.weekday().num_days_from_monday() != 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "period_begin_date must be a Monday".to_string(),
        ));
    }

    // Create a new schedule job
    let job_id = Uuid::new_v4();
    let now = Utc::now();

    let job = ScheduleJob {
        id: job_id,
        staff_group_id: request.staff_group_id,
        period_begin_date: request.period_begin_date,
        status: JobStatus::Pending,
        error_message: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
    };

    // Save to database
    let created_job = state
        .job_repo
        .create(job)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Submit to background processor
    let schedule_request = ScheduleJobRequest {
        job_id: created_job.id,
        staff_group_id: created_job.staff_group_id,
        period_begin_date: created_job.period_begin_date,
    };

    state
        .schedule_sender
        .send(schedule_request)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to submit job: {}", e),
            )
        })?;

    let response = ScheduleJobSerialize::from(created_job);

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// Get schedule job status
#[utoipa::path(
    get,
    path = "/api/v1/schedules/{schedule_id}/status",
    params(
        ("schedule_id" = Uuid, Path, description = "Schedule job ID")
    ),
    responses(
        (status = 200, description = "Schedule status retrieved", body = ScheduleStatusSerialize),
        (status = 404, description = "Schedule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "schedules"
)]
pub async fn get_schedule_status(
    State(state): State<AppState>,
    Path(schedule_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let job = state
        .job_repo
        .find_by_id(schedule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Schedule not found".to_string()))?;

    let response = ScheduleStatusSerialize::from(job);

    Ok((StatusCode::OK, Json(response)))
}

/// Get schedule result
#[utoipa::path(
    get,
    path = "/api/v1/schedules/{schedule_id}",
    params(
        ("schedule_id" = Uuid, Path, description = "Schedule job ID")
    ),
    responses(
        (status = 200, description = "Schedule result retrieved", body = ScheduleResultSerialize),
        (status = 404, description = "Schedule not found"),
        (status = 400, description = "Schedule not completed yet"),
        (status = 500, description = "Internal server error")
    ),
    tag = "schedules"
)]
pub async fn get_schedule_result(
    State(state): State<AppState>,
    Path(schedule_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get the job
    let job = state
        .job_repo
        .find_by_id(schedule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Schedule not found".to_string()))?;

    // Check if job is completed
    if job.status != JobStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Schedule is not completed yet. Current status: {:?}",
                job.status
            ),
        ));
    }

    // Get assignments
    let assignments = state
        .assignment_repo
        .find_by_job_id(schedule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let assignment_responses: Vec<ShiftAssignmentSerialize> =
        assignments.into_iter().map(|a| a.into()).collect();

    let response = ScheduleResultSerialize {
        schedule_id: job.id,
        period_begin_date: job.period_begin_date,
        staff_group_id: job.staff_group_id,
        assignments: assignment_responses,
    };

    Ok((StatusCode::OK, Json(response)))
}

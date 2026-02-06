use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Utc};
use shared::{ApiResponse, JobStatus};
use uuid::Uuid;

use crate::api::requests::CreateScheduleRequest;
use crate::api::requests::schedule_request::ScheduleJobRequest;
use crate::api::state::AppState;
use crate::domain::entities::ScheduleJob;
use crate::presentation::{
    ScheduleJobSerializer, ScheduleResultSerializer, ScheduleStatusSerializer,
    ShiftAssignmentSerializer,
};

#[utoipa::path(
    post,
    path = "/api/v1/schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 202, description = "Schedule job accepted for processing", body = ApiResponse<ScheduleJobSerializer>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "schedules"
)]
pub async fn submit_schedule(
    State(state): State<AppState>,
    Json(request): Json<CreateScheduleRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if request.period_begin_date.weekday().num_days_from_monday() != 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "period_begin_date must be a Monday".to_string(),
        ));
    }

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

    let created_job = state
        .job_repo
        .create(job)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiResponse::success(
            "Schedule job accepted for processing",
            ScheduleJobSerializer::from(created_job),
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/schedules/{schedule_id}/status",
    params(
        ("schedule_id" = Uuid, Path, description = "Schedule job ID")
    ),
    responses(
        (status = 200, description = "Schedule status retrieved", body = ApiResponse<ScheduleStatusSerializer>),
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

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Schedule status retrieved successfully",
            ScheduleStatusSerializer::from(job),
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/schedules/{schedule_id}",
    params(
        ("schedule_id" = Uuid, Path, description = "Schedule job ID")
    ),
    responses(
        (status = 200, description = "Schedule result retrieved", body = ApiResponse<ScheduleResultSerializer>),
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
    let job = state
        .job_repo
        .find_by_id(schedule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Schedule not found".to_string()))?;

    if job.status != JobStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Schedule is not completed yet. Current status: {:?}",
                job.status
            ),
        ));
    }

    let assignments = state
        .assignment_repo
        .find_by_job_id(schedule_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let assignment_responses: Vec<ShiftAssignmentSerializer> =
        assignments.into_iter().map(|a| a.into()).collect();

    let data = ScheduleResultSerializer {
        schedule_id: job.id,
        period_begin_date: job.period_begin_date,
        staff_group_id: job.staff_group_id,
        assignments: assignment_responses,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Schedule result retrieved successfully",
            data,
        )),
    ))
}

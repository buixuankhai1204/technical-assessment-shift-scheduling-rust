use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::state::AppState;
use crate::domain::entities::{CreateGroupRequest, CreateStaffRequest};

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportStaffRequest {
    pub staff: Vec<CreateStaffRequest>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchImportGroupsRequest {
    pub groups: Vec<CreateGroupRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportResponse {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Batch import staff from JSON
#[utoipa::path(
    post,
    path = "/api/v1/batch/staff",
    request_body = BatchImportStaffRequest,
    responses(
        (status = 200, description = "Batch import completed", body = BatchImportResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_staff(
    State(state): State<AppState>,
    Json(request): Json<BatchImportStaffRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for staff_request in request.staff {
        match state.staff_repo.create(staff_request).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(e.to_string());
            }
        }
    }

    // Invalidate cache
    let mut redis_conn = state.redis_pool.clone();
    let _: Result<(), _> = redis_conn.del("staff:list:*").await;

    let response = BatchImportResponse {
        success_count,
        error_count,
        errors,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Batch import groups from JSON
#[utoipa::path(
    post,
    path = "/api/v1/batch/groups",
    request_body = BatchImportGroupsRequest,
    responses(
        (status = 200, description = "Batch import completed", body = BatchImportResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_groups(
    State(state): State<AppState>,
    Json(request): Json<BatchImportGroupsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for group_request in request.groups {
        match state.group_repo.create(group_request).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(e.to_string());
            }
        }
    }

    // Invalidate cache
    let mut redis_conn = state.redis_pool.clone();
    let _: Result<(), _> = redis_conn.del("group:list:*").await;

    let response = BatchImportResponse {
        success_count,
        error_count,
        errors,
    };

    Ok((StatusCode::OK, Json(response)))
}

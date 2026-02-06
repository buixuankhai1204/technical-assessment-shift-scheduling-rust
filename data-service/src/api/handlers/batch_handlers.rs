use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use redis::AsyncCommands;
use serde::Serialize;
use shared::ApiResponse;
use utoipa::ToSchema;

use crate::api::requests::{CreateGroupRequest, CreateStaffRequest};
use crate::api::state::AppState;

/// Response for batch import operations
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportSerializer {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Batch import staff from sample-data/staff.json
#[utoipa::path(
    post,
    path = "/api/v1/batch/staff",
    responses(
        (status = 200, description = "Batch import completed", body = ApiResponse<BatchImportSerializer>),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_staff(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Read the staff.json file from sample-data folder
    let file_path = "sample-data/staff.json";
    let file_content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse JSON
    let staff_list: Vec<CreateStaffRequest> = serde_json::from_str(&file_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse JSON: {}", e),
        )
    })?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for staff_request in staff_list {
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

    let data = BatchImportSerializer {
        success_count,
        error_count,
        errors,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Batch staff import completed", data)),
    ))
}

/// Batch import groups from sample-data/groups.json
#[utoipa::path(
    post,
    path = "/api/v1/batch/groups",
    responses(
        (status = 200, description = "Batch import completed", body = BatchImportSerializer),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_groups(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Read the groups.json file from sample-data folder
    let file_path = "sample-data/groups.json";
    let file_content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;

    // Parse JSON
    let groups_list: Vec<CreateGroupRequest> =
        serde_json::from_str(&file_content).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse JSON: {}", e),
            )
        })?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for group_request in groups_list {
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

    let response = BatchImportSerializer {
        success_count,
        error_count,
        errors,
    };

    Ok((StatusCode::OK, Json(response)))
}

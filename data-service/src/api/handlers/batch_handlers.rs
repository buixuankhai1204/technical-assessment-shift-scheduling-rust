use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::ApiResponse;
use utoipa::ToSchema;

use crate::api::requests::CreateGroupRequest;
use crate::api::requests::CreateStaffRequest;
use crate::api::state::AppState;

const STAFF_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../sample-data/staff.json"));
const GROUPS_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../sample-data/groups.json"));
const MEMBERSHIPS_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../sample-data/memberships.json"));

#[derive(Debug, Deserialize)]
struct BatchGroupEntry {
    name: String,
    parent_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BatchMembershipEntry {
    staff_email: String,
    group_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchImportSerializer {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

async fn invalidate_cache_pattern(redis_conn: &mut redis::aio::ConnectionManager, pattern: &str) {
    let keys: Result<Vec<String>, _> = redis_conn.keys(pattern).await;
    if let Ok(keys) = keys {
        for key in keys {
            let _: Result<(), _> = redis_conn.del::<_, ()>(&key).await;
        }
    }
}

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
    let staff_list: Vec<CreateStaffRequest> = serde_json::from_str(STAFF_JSON).map_err(|e| {
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

    let mut redis_conn = state.redis_pool.clone();
    invalidate_cache_pattern(&mut redis_conn, "staff:*").await;

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

#[utoipa::path(
    post,
    path = "/api/v1/batch/groups",
    responses(
        (status = 200, description = "Batch import completed", body = ApiResponse<BatchImportSerializer>),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_groups(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let entries: Vec<BatchGroupEntry> = serde_json::from_str(GROUPS_JSON).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse JSON: {}", e),
        )
    })?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for entry in &entries {
        let request = CreateGroupRequest {
            name: entry.name.clone(),
            parent_id: None,
        };
        match state.group_repo.create(request).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(format!("Failed to create group '{}': {}", entry.name, e));
            }
        }
    }

    for entry in &entries {
        if let Some(parent_name) = &entry.parent_name {
            let parent = match state.group_repo.find_by_name(parent_name).await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    error_count += 1;
                    errors.push(format!(
                        "Parent group '{}' not found for '{}'",
                        parent_name, entry.name
                    ));
                    continue;
                }
                Err(e) => {
                    error_count += 1;
                    errors.push(format!(
                        "Error looking up parent '{}': {}",
                        parent_name, e
                    ));
                    continue;
                }
            };

            let child = match state.group_repo.find_by_name(&entry.name).await {
                Ok(Some(c)) => c,
                Ok(None) => continue,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!(
                        "Error looking up group '{}': {}",
                        entry.name, e
                    ));
                    continue;
                }
            };

            let update_request = crate::api::requests::UpdateGroupRequest {
                name: None,
                parent_id: Some(parent.id),
            };

            if let Err(e) = state.group_repo.update(child.id, update_request).await {
                error_count += 1;
                errors.push(format!(
                    "Failed to set parent for '{}': {}",
                    entry.name, e
                ));
            }
        }
    }

    let mut redis_conn = state.redis_pool.clone();
    invalidate_cache_pattern(&mut redis_conn, "group:*").await;

    let data = BatchImportSerializer {
        success_count,
        error_count,
        errors,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Batch groups import completed", data)),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/batch/memberships",
    responses(
        (status = 200, description = "Batch import completed", body = ApiResponse<BatchImportSerializer>),
        (status = 500, description = "Internal server error")
    ),
    tag = "batch"
)]
pub async fn batch_import_memberships(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let entries: Vec<BatchMembershipEntry> =
        serde_json::from_str(MEMBERSHIPS_JSON).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse JSON: {}", e),
            )
        })?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    for entry in &entries {
        let staff = match state.staff_repo.find_by_email(&entry.staff_email).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                error_count += 1;
                errors.push(format!("Staff with email '{}' not found", entry.staff_email));
                continue;
            }
            Err(e) => {
                error_count += 1;
                errors.push(format!(
                    "Error looking up staff '{}': {}",
                    entry.staff_email, e
                ));
                continue;
            }
        };

        let group = match state.group_repo.find_by_name(&entry.group_name).await {
            Ok(Some(g)) => g,
            Ok(None) => {
                error_count += 1;
                errors.push(format!("Group '{}' not found", entry.group_name));
                continue;
            }
            Err(e) => {
                error_count += 1;
                errors.push(format!(
                    "Error looking up group '{}': {}",
                    entry.group_name, e
                ));
                continue;
            }
        };

        match state
            .membership_repo
            .add_member(staff.id, group.id)
            .await
        {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(format!(
                    "Failed to add '{}' to '{}': {}",
                    entry.staff_email, entry.group_name, e
                ));
            }
        }
    }

    let mut redis_conn = state.redis_pool.clone();
    invalidate_cache_pattern(&mut redis_conn, "group:resolved:*").await;

    let data = BatchImportSerializer {
        success_count,
        error_count,
        errors,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Batch memberships import completed",
            data,
        )),
    ))
}

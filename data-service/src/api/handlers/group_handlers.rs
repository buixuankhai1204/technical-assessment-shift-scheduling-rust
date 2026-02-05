use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use redis::AsyncCommands;
use shared::{DomainError, PaginatedResponse, PaginationParams};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::presentation::{CreateGroupRequest, GroupResponse, UpdateGroupRequest};

const GROUP_CACHE_TTL: u64 = 300; // 5 minutes

/// Create a new staff group
#[utoipa::path(
    post,
    path = "/api/v1/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created successfully", body = GroupResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn create_group(
    State(state): State<AppState>,
    Json(request): Json<CreateGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let group = state
        .group_repo
        .create(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Invalidate cache
    let mut redis_conn = state.redis_pool.clone();
    let _: Result<(), _> = redis_conn.del("group:list:*").await;

    Ok((StatusCode::CREATED, Json(GroupResponse::from(group))))
}

/// Get group by ID
#[utoipa::path(
    get,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group found", body = GroupResponse),
        (status = 404, description = "Group not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn get_group_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = format!("group:id:{}", id);
    let mut redis_conn = state.redis_pool.clone();

    // Try cache first
    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(group_response) = serde_json::from_str::<GroupResponse>(&cached_data) {
            return Ok((StatusCode::OK, Json(group_response)));
        }
    }

    // Fetch from database
    let group = state
        .group_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Group not found".to_string()))?;

    let response = GroupResponse::from(group);

    // Cache the result
    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

/// List all groups with pagination
#[utoipa::path(
    get,
    path = "/api/v1/groups",
    params(PaginationParams),
    responses(
        (status = 200, description = "Group list", body = PaginatedResponse<GroupResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn list_groups(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = format!("group:list:{}:{}", params.page, params.page_size);
    let mut redis_conn = state.redis_pool.clone();

    // Try cache first
    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(response) = serde_json::from_str::<PaginatedResponse<GroupResponse>>(&cached_data)
        {
            return Ok((StatusCode::OK, Json(response)));
        }
    }

    // Fetch from database
    let (groups, total) = state
        .group_repo
        .list(params.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = PaginatedResponse::new(
        groups.into_iter().map(GroupResponse::from).collect(),
        params.page,
        params.page_size,
        total,
    );

    // Cache the result
    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

/// Update group by ID
#[utoipa::path(
    put,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "Group updated successfully", body = GroupResponse),
        (status = 404, description = "Group not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn update_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let group = state
        .group_repo
        .update(id, request)
        .await
        .map_err(|e| match e {
            DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    // Invalidate cache
    let mut redis_conn = state.redis_pool.clone();
    let cache_key = format!("group:id:{}", id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;
    let _: Result<(), _> = redis_conn.del("group:list:*").await;
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", id)).await;

    Ok((StatusCode::OK, Json(GroupResponse::from(group))))
}

/// Delete group by ID
#[utoipa::path(
    delete,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 204, description = "Group deleted successfully"),
        (status = 404, description = "Group not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn delete_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state.group_repo.delete(id).await.map_err(|e| match e {
        DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    // Invalidate cache
    let mut redis_conn = state.redis_pool.clone();
    let cache_key = format!("group:id:{}", id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;
    let _: Result<(), _> = redis_conn.del("group:list:*").await;
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", id)).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Get all members in a group, including members of nested subgroups (hierarchical resolution)
#[utoipa::path(
    get,
    path = "/api/v1/groups/{id}/resolved-members",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Resolved members", body = Vec<crate::domain::entities::StaffResponse>),
        (status = 404, description = "Group not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn get_resolved_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = format!("group:resolved:{}", id);
    let mut redis_conn = state.redis_pool.clone();

    // Try cache first
    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(staff_list) =
            serde_json::from_str::<Vec<crate::domain::entities::StaffResponse>>(&cached_data)
        {
            return Ok((StatusCode::OK, Json(staff_list)));
        }
    }

    // Fetch from database
    let staff_list = state
        .group_service
        .get_resolved_members(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<crate::domain::entities::StaffResponse> = staff_list
        .into_iter()
        .map(crate::domain::entities::StaffResponse::from)
        .collect();

    // Cache the result
    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

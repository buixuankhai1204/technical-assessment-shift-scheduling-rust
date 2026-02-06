use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use redis::AsyncCommands;
use shared::{ApiResponse, DomainError, PaginationParams};
use uuid::Uuid;

use crate::api::requests::{CreateGroupRequest, UpdateGroupRequest};
use crate::api::state::AppState;
use crate::domain::entities::StaffGroup;
use crate::presentation::{GroupSerializer, ResolvedGroupSerializer};

const GROUP_CACHE_TTL: u64 = 300;

async fn invalidate_cache_pattern(redis_conn: &mut redis::aio::ConnectionManager, pattern: &str) {
    let keys: Result<Vec<String>, _> = redis_conn.keys(pattern).await;
    if let Ok(keys) = keys {
        for key in keys {
            let _: Result<(), _> = redis_conn.del::<_, ()>(&key).await;
        }
    }
}

async fn resolve_parent_name(
    state: &AppState,
    group: &StaffGroup,
) -> Result<Option<String>, (StatusCode, String)> {
    if let Some(parent_id) = group.parent_id {
        let parent = state
            .group_repo
            .find_by_id(parent_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(parent.map(|p| p.name))
    } else {
        Ok(None)
    }
}

async fn to_group_serializer(
    state: &AppState,
    group: StaffGroup,
) -> Result<GroupSerializer, (StatusCode, String)> {
    let parent_name = resolve_parent_name(state, &group).await?;
    Ok(GroupSerializer::new(group, parent_name))
}

#[utoipa::path(
    post,
    path = "/api/v1/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created successfully", body = ApiResponse<GroupSerializer>),
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

    let mut redis_conn = state.redis_pool.clone();
    invalidate_cache_pattern(&mut redis_conn, "group:list:*").await;

    let serializer = to_group_serializer(&state, group).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Group created successfully",
            serializer,
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group found", body = ApiResponse<GroupSerializer>),
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

    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(group_response) =
            serde_json::from_str::<ApiResponse<GroupSerializer>>(&cached_data)
        {
            return Ok((StatusCode::OK, Json(group_response)));
        }
    }

    let group = state
        .group_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Group not found".to_string()))?;

    let serializer = to_group_serializer(&state, group).await?;
    let response = ApiResponse::success("Group retrieved successfully", serializer);

    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/groups",
    params(PaginationParams),
    responses(
        (status = 200, description = "Group list", body = ApiResponse<Vec<GroupSerializer>>),
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

    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(response) =
            serde_json::from_str::<ApiResponse<Vec<GroupSerializer>>>(&cached_data)
        {
            return Ok((StatusCode::OK, Json(response)));
        }
    }

    let (groups, total) = state
        .group_repo
        .list(params.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut serialized = Vec::with_capacity(groups.len());
    for group in groups {
        serialized.push(to_group_serializer(&state, group).await?);
    }

    let response = ApiResponse::with_total("Group list retrieved successfully", serialized, total);

    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    put,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "Group updated successfully", body = ApiResponse<GroupSerializer>),
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

    let mut redis_conn = state.redis_pool.clone();
    let cache_key = format!("group:id:{}", id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;
    invalidate_cache_pattern(&mut redis_conn, "group:list:*").await;
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", id)).await;

    let serializer = to_group_serializer(&state, group).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Group updated successfully",
            serializer,
        )),
    ))
}

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

    let mut redis_conn = state.redis_pool.clone();
    let cache_key = format!("group:id:{}", id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;
    invalidate_cache_pattern(&mut redis_conn, "group:list:*").await;
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", id)).await;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/groups/{id}/resolved-members",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Resolved members grouped by subgroup", body = ApiResponse<Vec<ResolvedGroupSerializer>>),
        (status = 404, description = "Group not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "groups"
)]
pub async fn get_resolved_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state
        .group_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, format!("Group with id {} not found", id)))?;

    let cache_key = format!("group:resolved:{}", id);
    let mut redis_conn = state.redis_pool.clone();

    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_data) = cached {
        if let Ok(response) =
            serde_json::from_str::<ApiResponse<Vec<ResolvedGroupSerializer>>>(&cached_data)
        {
            return Ok((StatusCode::OK, Json(response)));
        }
    }

    let (groups_with_members, total_unique) = state
        .group_repo
        .get_resolved_members(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let serialized: Vec<ResolvedGroupSerializer> = groups_with_members
        .into_iter()
        .map(ResolvedGroupSerializer::from)
        .collect();

    let response = ApiResponse::with_total(
        "Resolved members retrieved successfully",
        serialized,
        total_unique,
    );

    let _: Result<(), _> = redis_conn
        .set_ex(
            &cache_key,
            serde_json::to_string(&response).unwrap(),
            GROUP_CACHE_TTL,
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
}

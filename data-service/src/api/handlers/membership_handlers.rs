use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use redis::AsyncCommands;
use shared::DomainError;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::presentation::{AddMemberRequest, MembershipResponse};

/// Add staff to group
#[utoipa::path(
    post,
    path = "/api/v1/groups/{group_id}/members",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added successfully", body = MembershipResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "memberships"
)]
pub async fn add_member(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let membership = state
        .membership_repo
        .add_member(request.staff_id, group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Invalidate cache for resolved members
    let mut redis_conn = state.redis_pool.clone();
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", group_id)).await;

    Ok((
        StatusCode::CREATED,
        Json(MembershipResponse::from(membership)),
    ))
}

/// Remove staff from group
#[utoipa::path(
    delete,
    path = "/api/v1/groups/{group_id}/members/{staff_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("staff_id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 204, description = "Member removed successfully"),
        (status = 404, description = "Membership not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "memberships"
)]
pub async fn remove_member(
    State(state): State<AppState>,
    Path((group_id, staff_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state
        .membership_repo
        .remove_member(staff_id, group_id)
        .await
        .map_err(|e| match e {
            DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    // Invalidate cache for resolved members
    let mut redis_conn = state.redis_pool.clone();
    let _: Result<(), _> = redis_conn.del(format!("group:resolved:{}", group_id)).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Get all members of a group (direct members only, not hierarchical)
#[utoipa::path(
    get,
    path = "/api/v1/groups/{group_id}/members",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group members", body = Vec<crate::domain::entities::StaffResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "memberships"
)]
pub async fn get_group_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let staff_list = state
        .staff_repo
        .find_by_group_id(group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<crate::domain::entities::StaffResponse> = staff_list
        .into_iter()
        .map(crate::domain::entities::StaffResponse::from)
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

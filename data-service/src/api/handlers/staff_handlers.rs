use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use shared::{ApiResponse, DomainError, PaginationParams};
use uuid::Uuid;

use crate::api::requests::{CreateStaffRequest, UpdateStaffRequest};
use crate::api::state::AppState;
use crate::presentation::StaffSerializer;

#[utoipa::path(
    post,
    path = "/api/v1/staff",
    request_body = CreateStaffRequest,
    responses(
        (status = 201, description = "Staff created successfully", body = ApiResponse<StaffSerializer>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "staff"
)]
pub async fn create_staff(
    State(state): State<AppState>,
    Json(request): Json<CreateStaffRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let staff = state
        .staff_repo
        .create(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Staff created successfully",
            StaffSerializer::from(staff),
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 200, description = "Staff found", body = ApiResponse<StaffSerializer>),
        (status = 404, description = "Staff not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "staff"
)]
pub async fn get_staff_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let staff = state
        .staff_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Staff not found".to_string()))?;

    let response =
        ApiResponse::success("Staff retrieved successfully", StaffSerializer::from(staff));

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/staff",
    params(PaginationParams),
    responses(
        (status = 200, description = "Staff list", body = ApiResponse<Vec<StaffSerializer>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "staff"
)]
pub async fn list_staff(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (staff_list, total) = state
        .staff_repo
        .list(params.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let serialized: Vec<StaffSerializer> =
        staff_list.into_iter().map(StaffSerializer::from).collect();

    let response = ApiResponse::with_total("Staff list retrieved successfully", serialized, total);

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    put,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    request_body = UpdateStaffRequest,
    responses(
        (status = 200, description = "Staff updated successfully", body = ApiResponse<StaffSerializer>),
        (status = 404, description = "Staff not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "staff"
)]
pub async fn update_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateStaffRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let staff = state
        .staff_repo
        .update(id, request)
        .await
        .map_err(|e| match e {
            DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Staff updated successfully",
            StaffSerializer::from(staff),
        )),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 204, description = "Staff deleted successfully"),
        (status = 404, description = "Staff not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "staff"
)]
pub async fn delete_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state.staff_repo.delete(id).await.map_err(|e| match e {
        DomainError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    Ok(StatusCode::NO_CONTENT)
}

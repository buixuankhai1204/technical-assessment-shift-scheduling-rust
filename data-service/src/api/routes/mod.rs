use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{handlers, state::AppState};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Data Service API",
        version = "1.0.0",
        description = "Staff and group management API with Redis caching"
    ),
    paths(
        // Staff endpoints
        handlers::staff_handlers::create_staff,
        handlers::staff_handlers::get_staff_by_id,
        handlers::staff_handlers::list_staff,
        handlers::staff_handlers::update_staff,
        handlers::staff_handlers::delete_staff,
        // Group endpoints
        handlers::group_handlers::create_group,
        handlers::group_handlers::get_group_by_id,
        handlers::group_handlers::list_groups,
        handlers::group_handlers::update_group,
        handlers::group_handlers::delete_group,
        handlers::group_handlers::get_resolved_members,
        // Membership endpoints
        handlers::membership_handlers::add_member,
        handlers::membership_handlers::remove_member,
        handlers::membership_handlers::get_group_members,
        // Batch import endpoints
        handlers::batch_handlers::batch_import_staff,
        handlers::batch_handlers::batch_import_groups,
    ),
    components(schemas(
        // Shared types
        shared::StaffStatus,
        shared::PaginationParams,
        shared::PaginatedResponse<crate::domain::entities::StaffResponse>,
        shared::PaginatedResponse<crate::domain::entities::GroupResponse>,
        // Staff schemas
        crate::domain::entities::Staff,
        crate::domain::entities::StaffResponse,
        crate::domain::entities::CreateStaffRequest,
        crate::domain::entities::UpdateStaffRequest,
        // Group schemas
        crate::domain::entities::StaffGroup,
        crate::domain::entities::GroupResponse,
        crate::domain::entities::CreateGroupRequest,
        crate::domain::entities::UpdateGroupRequest,
        // Membership schemas
        crate::domain::entities::GroupMembership,
        crate::domain::entities::MembershipResponse,
        crate::domain::entities::AddMemberRequest,
        crate::domain::entities::RemoveMemberRequest,
        // Batch import schemas
        crate::api::handlers::batch_handlers::BatchImportStaffRequest,
        crate::api::handlers::batch_handlers::BatchImportGroupsRequest,
        crate::api::handlers::batch_handlers::BatchImportResponse,
    )),
    tags(
        (name = "staff", description = "Staff management endpoints"),
        (name = "groups", description = "Group management endpoints"),
        (name = "memberships", description = "Group membership management endpoints"),
        (name = "batch", description = "Batch import endpoints")
    )
)]
struct ApiDoc;

pub fn create_router(app_state: AppState) -> Router {
    let staff_routes = Router::new()
        .route("/staff", post(handlers::staff_handlers::create_staff))
        .route("/staff", get(handlers::staff_handlers::list_staff))
        .route("/staff/:id", get(handlers::staff_handlers::get_staff_by_id))
        .route("/staff/:id", put(handlers::staff_handlers::update_staff))
        .route("/staff/:id", delete(handlers::staff_handlers::delete_staff));

    let group_routes = Router::new()
        .route("/groups", post(handlers::group_handlers::create_group))
        .route("/groups", get(handlers::group_handlers::list_groups))
        .route(
            "/groups/:id",
            get(handlers::group_handlers::get_group_by_id),
        )
        .route("/groups/:id", put(handlers::group_handlers::update_group))
        .route(
            "/groups/:id",
            delete(handlers::group_handlers::delete_group),
        )
        .route(
            "/groups/:id/resolved-members",
            get(handlers::group_handlers::get_resolved_members),
        );

    let membership_routes = Router::new()
        .route(
            "/groups/:group_id/members",
            post(handlers::membership_handlers::add_member),
        )
        .route(
            "/groups/:group_id/members",
            get(handlers::membership_handlers::get_group_members),
        )
        .route(
            "/groups/:group_id/members/:staff_id",
            delete(handlers::membership_handlers::remove_member),
        );

    let batch_routes = Router::new()
        .route(
            "/batch/staff",
            post(handlers::batch_handlers::batch_import_staff),
        )
        .route(
            "/batch/groups",
            post(handlers::batch_handlers::batch_import_groups),
        );

    let api_router = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(staff_routes)
        .merge(group_routes)
        .merge(membership_routes)
        .merge(batch_routes);

    Router::new()
        .nest("/api/v1", api_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

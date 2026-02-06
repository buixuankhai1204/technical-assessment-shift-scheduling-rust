use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::handlers;
use crate::api::state::AppState;
use crate::presentation::{
    ScheduleJobSerialize, ScheduleResultSerialize, ScheduleStatusSerialize,
    ShiftAssignmentSerialize,
};
use shared::{JobStatus, ShiftType};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Scheduling Service API",
        version = "1.0.0",
        description = "Asynchronous shift schedule generation API"
    ),
    paths(
        crate::api::handlers::schedule_handlers::submit_schedule,
        crate::api::handlers::schedule_handlers::get_schedule_status,
        crate::api::handlers::schedule_handlers::get_schedule_result,
    ),
    components(schemas(
        crate::api::requests::CreateScheduleRequest,
        ScheduleJobSerialize,
        ScheduleStatusSerialize,
        ScheduleResultSerialize,
        ShiftAssignmentSerialize,
        JobStatus,
        ShiftType,
    ))
)]
struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/schedules", post(handlers::submit_schedule))
        .route(
            "/schedules/:schedule_id/status",
            get(handlers::get_schedule_status),
        )
        .route(
            "/schedules/:schedule_id",
            get(handlers::get_schedule_result),
        );

    Router::new()
        .nest("/api/v1", api_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

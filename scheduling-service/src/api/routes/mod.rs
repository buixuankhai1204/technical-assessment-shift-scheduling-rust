use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::handlers;
use crate::infrastructure::database::DbPool;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Scheduling Service API",
        version = "1.0.0",
        description = "Asynchronous shift schedule generation API"
    ),
    paths(),
    components(schemas())
)]
struct ApiDoc;

pub fn create_router(db_pool: DbPool) -> Router {
    let api_router = Router::new().route("/health", get(handlers::health_check));

    Router::new()
        .nest("/api/v1", api_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool)
}

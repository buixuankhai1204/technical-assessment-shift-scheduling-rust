// API handlers will be implemented here
// Example: staff_handlers.rs, group_handlers.rs

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

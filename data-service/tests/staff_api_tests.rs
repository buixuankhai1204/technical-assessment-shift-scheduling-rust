//! Staff API integration tests

#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use common::{
    create_mock_redis_pool, create_sample_staff, create_test_app_state, MockGroupRepository,
    MockMembershipRepository, MockStaffRepository,
};
use data_service::api::create_router;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_server() -> TestServer {
    let staff_repo = Arc::new(MockStaffRepository::new());
    let group_repo = Arc::new(MockGroupRepository::new());
    let membership_repo = Arc::new(MockMembershipRepository::new());
    let redis_pool = create_mock_redis_pool().await;

    let state = create_test_app_state(staff_repo, group_repo, membership_repo, redis_pool);
    let app = create_router(state);

    TestServer::new(app).unwrap()
}

async fn setup_test_server_with_staff(
    staff_list: Vec<data_service::domain::entities::Staff>,
) -> TestServer {
    let staff_repo = Arc::new(MockStaffRepository::with_staff(staff_list));
    let group_repo = Arc::new(MockGroupRepository::new());
    let membership_repo = Arc::new(MockMembershipRepository::new());
    let redis_pool = create_mock_redis_pool().await;

    let state = create_test_app_state(staff_repo, group_repo, membership_repo, redis_pool);
    let app = create_router(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let server = setup_test_server().await;

    let response = server.get("/api/v1/health").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_create_staff_success() {
    let server = setup_test_server().await;

    let request_body = json!({
        "name": "John Doe",
        "email": "john.doe@example.com",
        "position": "Software Engineer"
    });

    let response = server.post("/api/v1/staff").json(&request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Staff created successfully");
    assert_eq!(body["data"]["name"], "John Doe");
    assert_eq!(body["data"]["email"], "john.doe@example.com");
    assert_eq!(body["data"]["position"], "Software Engineer");
    assert_eq!(body["data"]["status"], "ACTIVE");
}

#[tokio::test]
async fn test_create_staff_with_status() {
    let server = setup_test_server().await;

    let request_body = json!({
        "name": "Jane Doe",
        "email": "jane.doe@example.com",
        "position": "Manager",
        "status": "INACTIVE"
    });

    let response = server.post("/api/v1/staff").json(&request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["status"], "INACTIVE");
}

#[tokio::test]
async fn test_get_staff_by_id_success() {
    let staff_id = Uuid::new_v4();
    let staff = create_sample_staff(staff_id, "John Doe", "john@example.com");
    let server = setup_test_server_with_staff(vec![staff]).await;

    let response = server.get(&format!("/api/v1/staff/{}", staff_id)).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Staff retrieved successfully");
    assert_eq!(body["data"]["name"], "John Doe");
}

#[tokio::test]
async fn test_get_staff_by_id_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response = server
        .get(&format!("/api/v1/staff/{}", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "Skipped due to Redis cache interference in parallel test execution"]
async fn test_list_staff_empty() {
    let server = setup_test_server().await;

    let response = server.get("/api/v1/staff").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Staff list retrieved successfully");
    // Note: In integration tests with shared Redis, cache may contain data from other tests
}

#[tokio::test]
async fn test_list_staff_with_data() {
    let staff1 = create_sample_staff(Uuid::new_v4(), "John Doe", "john@example.com");
    let staff2 = create_sample_staff(Uuid::new_v4(), "Jane Doe", "jane@example.com");
    let server = setup_test_server_with_staff(vec![staff1, staff2]).await;

    let response = server.get("/api/v1/staff").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["total"], 2);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_staff_with_pagination() {
    let staff1 = create_sample_staff(Uuid::new_v4(), "Staff 1", "staff1@example.com");
    let staff2 = create_sample_staff(Uuid::new_v4(), "Staff 2", "staff2@example.com");
    let staff3 = create_sample_staff(Uuid::new_v4(), "Staff 3", "staff3@example.com");
    let server = setup_test_server_with_staff(vec![staff1, staff2, staff3]).await;

    let response = server.get("/api/v1/staff?page=1&page_size=2").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["total"], 3);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_staff_success() {
    let staff_id = Uuid::new_v4();
    let staff = create_sample_staff(staff_id, "John Doe", "john@example.com");
    let server = setup_test_server_with_staff(vec![staff]).await;

    let update_request = json!({
        "name": "John Updated",
        "position": "Senior Engineer"
    });

    let response = server
        .put(&format!("/api/v1/staff/{}", staff_id))
        .json(&update_request)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["name"], "John Updated");
    assert_eq!(body["data"]["position"], "Senior Engineer");
}

#[tokio::test]
async fn test_update_staff_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let update_request = json!({
        "name": "Updated Name"
    });

    let response = server
        .put(&format!("/api/v1/staff/{}", non_existent_id))
        .json(&update_request)
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_staff_success() {
    let staff_id = Uuid::new_v4();
    let staff = create_sample_staff(staff_id, "John Doe", "john@example.com");
    let server = setup_test_server_with_staff(vec![staff]).await;

    let response = server.delete(&format!("/api/v1/staff/{}", staff_id)).await;

    response.assert_status(StatusCode::NO_CONTENT);

    // Verify staff is deleted
    let get_response = server.get(&format!("/api/v1/staff/{}", staff_id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_staff_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response = server
        .delete(&format!("/api/v1/staff/{}", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

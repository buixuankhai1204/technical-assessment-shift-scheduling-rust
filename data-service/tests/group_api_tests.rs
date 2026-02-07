//! Group API integration tests

#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use common::{
    create_mock_redis_pool, create_sample_group, create_test_app_state, MockGroupRepository,
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

async fn setup_test_server_with_groups(
    group_list: Vec<data_service::domain::entities::StaffGroup>,
) -> TestServer {
    let staff_repo = Arc::new(MockStaffRepository::new());
    let group_repo = Arc::new(MockGroupRepository::with_groups(group_list));
    let membership_repo = Arc::new(MockMembershipRepository::new());
    let redis_pool = create_mock_redis_pool().await;

    let state = create_test_app_state(staff_repo, group_repo, membership_repo, redis_pool);
    let app = create_router(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_create_group_success() {
    let server = setup_test_server().await;

    let request_body = json!({
        "name": "Engineering Team"
    });

    let response = server.post("/api/v1/groups").json(&request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Group created successfully");
    assert_eq!(body["data"]["name"], "Engineering Team");
    assert!(body["data"]["parent_id"].is_null());
}

#[tokio::test]
async fn test_create_group_with_parent() {
    let parent_id = Uuid::new_v4();
    let parent_group = create_sample_group(parent_id, "Parent Group", None);
    let server = setup_test_server_with_groups(vec![parent_group]).await;

    let request_body = json!({
        "name": "Child Group",
        "parent_id": parent_id.to_string()
    });

    let response = server.post("/api/v1/groups").json(&request_body).await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["name"], "Child Group");
    assert_eq!(body["data"]["parent_id"], parent_id.to_string());
}

#[tokio::test]
async fn test_get_group_by_id_success() {
    let group_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Test Group", None);
    let server = setup_test_server_with_groups(vec![group]).await;

    let response = server.get(&format!("/api/v1/groups/{}", group_id)).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Group retrieved successfully");
    assert_eq!(body["data"]["name"], "Test Group");
}

#[tokio::test]
async fn test_get_group_by_id_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response = server
        .get(&format!("/api/v1/groups/{}", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_groups_empty() {
    let server = setup_test_server().await;

    let response = server.get("/api/v1/groups").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Group list retrieved successfully");
    assert!(body["data"].as_array().unwrap().is_empty());
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn test_list_groups_with_data() {
    let group1 = create_sample_group(Uuid::new_v4(), "Group 1", None);
    let group2 = create_sample_group(Uuid::new_v4(), "Group 2", None);
    let server = setup_test_server_with_groups(vec![group1, group2]).await;

    let response = server.get("/api/v1/groups").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["total"], 2);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_groups_with_pagination() {
    let group1 = create_sample_group(Uuid::new_v4(), "Group 1", None);
    let group2 = create_sample_group(Uuid::new_v4(), "Group 2", None);
    let group3 = create_sample_group(Uuid::new_v4(), "Group 3", None);
    let server = setup_test_server_with_groups(vec![group1, group2, group3]).await;

    let response = server.get("/api/v1/groups?page=1&page_size=2").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["total"], 3);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_group_success() {
    let group_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Original Name", None);
    let server = setup_test_server_with_groups(vec![group]).await;

    let update_request = json!({
        "name": "Updated Name"
    });

    let response = server
        .put(&format!("/api/v1/groups/{}", group_id))
        .json(&update_request)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["name"], "Updated Name");
}

#[tokio::test]
async fn test_update_group_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let update_request = json!({
        "name": "Updated Name"
    });

    let response = server
        .put(&format!("/api/v1/groups/{}", non_existent_id))
        .json(&update_request)
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_group_success() {
    let group_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Test Group", None);
    let server = setup_test_server_with_groups(vec![group]).await;

    let response = server.delete(&format!("/api/v1/groups/{}", group_id)).await;

    response.assert_status(StatusCode::NO_CONTENT);

    // Verify group is deleted
    let get_response = server.get(&format!("/api/v1/groups/{}", group_id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_group_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response = server
        .delete(&format!("/api/v1/groups/{}", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_resolved_members_success() {
    let group_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Test Group", None);
    let server = setup_test_server_with_groups(vec![group]).await;

    let response = server
        .get(&format!("/api/v1/groups/{}/resolved-members", group_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Resolved members retrieved successfully");
}

#[tokio::test]
async fn test_get_resolved_members_not_found() {
    let server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response = server
        .get(&format!(
            "/api/v1/groups/{}/resolved-members",
            non_existent_id
        ))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

//! Membership API integration tests

#[path = "common/mod.rs"]
mod common;

use common::{
    create_mock_redis_pool, create_sample_group, create_sample_staff, create_test_app_state,
    MockGroupRepository, MockMembershipRepository, MockStaffRepository,
};
use axum::http::StatusCode;
use axum_test::TestServer;
use data_service::api::create_router;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_server_with_data(
    staff_list: Vec<data_service::domain::entities::Staff>,
    group_list: Vec<data_service::domain::entities::StaffGroup>,
) -> TestServer {
    let staff_repo = Arc::new(MockStaffRepository::with_staff(staff_list));
    let group_repo = Arc::new(MockGroupRepository::with_groups(group_list));
    let membership_repo = Arc::new(MockMembershipRepository::new());
    let redis_pool = create_mock_redis_pool().await;

    let state = create_test_app_state(staff_repo, group_repo, membership_repo, redis_pool);
    let app = create_router(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_add_member_success() {
    let staff_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let staff = create_sample_staff(staff_id, "John Doe", "john@example.com");
    let group = create_sample_group(group_id, "Engineering", None);

    let server = setup_test_server_with_data(vec![staff], vec![group]).await;

    let request_body = json!({
        "staff_id": staff_id.to_string()
    });

    let response = server
        .post(&format!("/api/v1/groups/{}/members", group_id))
        .json(&request_body)
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Member added successfully");
    assert_eq!(body["data"]["staff_id"], staff_id.to_string());
    assert_eq!(body["data"]["group_id"], group_id.to_string());
}

#[tokio::test]
async fn test_add_member_staff_not_found() {
    let group_id = Uuid::new_v4();
    let non_existent_staff_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Engineering", None);

    let server = setup_test_server_with_data(vec![], vec![group]).await;

    let request_body = json!({
        "staff_id": non_existent_staff_id.to_string()
    });

    let response = server
        .post(&format!("/api/v1/groups/{}/members", group_id))
        .json(&request_body)
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_add_member_group_not_found() {
    let staff_id = Uuid::new_v4();
    let non_existent_group_id = Uuid::new_v4();
    let staff = create_sample_staff(staff_id, "John Doe", "john@example.com");

    let server = setup_test_server_with_data(vec![staff], vec![]).await;

    let request_body = json!({
        "staff_id": staff_id.to_string()
    });

    let response = server
        .post(&format!("/api/v1/groups/{}/members", non_existent_group_id))
        .json(&request_body)
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_group_members_empty() {
    let group_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Engineering", None);

    let server = setup_test_server_with_data(vec![], vec![group]).await;

    let response = server
        .get(&format!("/api/v1/groups/{}/members", group_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Group members retrieved successfully");
    assert!(body["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_remove_member_not_found() {
    let group_id = Uuid::new_v4();
    let staff_id = Uuid::new_v4();
    let group = create_sample_group(group_id, "Engineering", None);

    let server = setup_test_server_with_data(vec![], vec![group]).await;

    let response = server
        .delete(&format!("/api/v1/groups/{}/members/{}", group_id, staff_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}


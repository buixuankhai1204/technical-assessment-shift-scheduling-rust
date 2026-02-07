#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use axum_test::{TestResponse, TestServer};
use common::{
    create_completed_job, create_sample_assignments, create_sample_job, create_test_app_state,
    get_test_monday, MockScheduleJobRepository, MockShiftAssignmentRepository,
    TestServerWithReceiver,
};
use scheduling_service::api::create_router;
use scheduling_service::domain::entities::{ScheduleJob, ShiftAssignment};
use scheduling_service::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};
use scheduling_service::domain::rules::{
    MaxDaysOffRule, MinDaysOffRule, NoMorningAfterEveningRule, ShiftBalanceRule,
};
use serde_json::json;
use shared::JobStatus;
use std::sync::Arc;
use uuid::Uuid;

/// Setup a test server with empty mock repositories
async fn setup_test_server() -> TestServerWithReceiver {
    let job_repo = Arc::new(MockScheduleJobRepository::new());
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::new());

    let (state, receiver) = create_test_app_state(job_repo, assignment_repo);
    let app = create_router(state);

    TestServerWithReceiver {
        server: TestServer::new(app).unwrap(),
        receiver,
    }
}

/// Setup a test server with pre-configured jobs and assignments
async fn setup_test_server_with_jobs(
    job_list: Vec<ScheduleJob>,
    assignment_list: Vec<ShiftAssignment>,
) -> TestServerWithReceiver {
    let job_repo = Arc::new(MockScheduleJobRepository::with_jobs(job_list));
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::with_assignments(
        assignment_list,
    ));

    let (state, receiver) = create_test_app_state(job_repo, assignment_repo);
    let app = create_router(state);

    TestServerWithReceiver {
        server: TestServer::new(app).unwrap(),
        receiver,
    }
}

#[tokio::test]
async fn test_submit_schedule_success() {
    let test_server = setup_test_server().await;
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();

    let request_body = json!({
        "staff_group_id": group_id.to_string(),
        "period_begin_date": monday.to_string()
    });

    let response: TestResponse = test_server
        .server
        .post("/api/v1/schedules")
        .json(&request_body)
        .await;

    response.assert_status(StatusCode::ACCEPTED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Schedule job accepted for processing");
    assert!(body["data"]["schedule_id"].is_string());
    assert_eq!(body["data"]["status"], "PENDING");
}

#[tokio::test]
async fn test_submit_schedule_invalid_date_not_monday() {
    let test_server = setup_test_server().await;
    let group_id = Uuid::new_v4();
    // Use a Tuesday instead of Monday
    let tuesday = get_test_monday() + chrono::Duration::days(1);

    let request_body = json!({
        "staff_group_id": group_id.to_string(),
        "period_begin_date": tuesday.to_string()
    });

    let response: TestResponse = test_server
        .server
        .post("/api/v1/schedules")
        .json(&request_body)
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_schedule_status_pending() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    let test_server = setup_test_server_with_jobs(vec![job], vec![]).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}/status", job_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Schedule status retrieved successfully");
    assert_eq!(body["data"]["status"], "PENDING");
}

#[tokio::test]
async fn test_get_schedule_status_processing() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Processing);

    let test_server = setup_test_server_with_jobs(vec![job], vec![]).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}/status", job_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["status"], "PROCESSING");
}

#[tokio::test]
async fn test_get_schedule_status_completed() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    let job = create_completed_job(job_id, group_id, monday);

    let test_server = setup_test_server_with_jobs(vec![job], vec![]).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}/status", job_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["status"], "COMPLETED");
}

#[tokio::test]
async fn test_get_schedule_status_not_found() {
    let test_server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}/status", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_schedule_result_success() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    let job = create_completed_job(job_id, group_id, monday);

    let staff_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    let assignments = create_sample_assignments(job_id, staff_ids, monday);

    let test_server = setup_test_server_with_jobs(vec![job], assignments).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}", job_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["message"], "Schedule result retrieved successfully");
    assert!(body["data"]["assignments"].is_array());
}

#[tokio::test]
async fn test_get_schedule_result_not_completed() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    // Job is still pending, not completed
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    let test_server = setup_test_server_with_jobs(vec![job], vec![]).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}", job_id))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_schedule_result_processing() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    // Job is still processing
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Processing);

    let test_server = setup_test_server_with_jobs(vec![job], vec![]).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}", job_id))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_schedule_result_not_found() {
    let test_server = setup_test_server().await;
    let non_existent_id = Uuid::new_v4();

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}", non_existent_id))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_submit_multiple_schedules() {
    let test_server: TestServerWithReceiver = setup_test_server().await;
    let group_id1 = Uuid::new_v4();
    let group_id2 = Uuid::new_v4();
    let monday = get_test_monday();

    // Submit first schedule
    let request1 = json!({
        "staff_group_id": group_id1.to_string(),
        "period_begin_date": monday.to_string()
    });

    let response1: TestResponse = test_server
        .server
        .post("/api/v1/schedules")
        .json(&request1)
        .await;

    response1.assert_status(StatusCode::ACCEPTED);

    // Submit second schedule
    let request2 = json!({
        "staff_group_id": group_id2.to_string(),
        "period_begin_date": monday.to_string()
    });

    let response2: TestResponse = test_server
        .server
        .post("/api/v1/schedules")
        .json(&request2)
        .await;

    response2.assert_status(StatusCode::ACCEPTED);

    // Verify they have different IDs
    let body1: serde_json::Value = response1.json();
    let body2: serde_json::Value = response2.json();
    assert_ne!(body1["data"]["schedule_id"], body2["data"]["schedule_id"]);
}

#[tokio::test]
async fn test_schedule_result_contains_expected_fields() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();
    let job = create_completed_job(job_id, group_id, monday);

    let staff_id = Uuid::new_v4();
    let assignments = create_sample_assignments(job_id, vec![staff_id], monday);

    let test_server = setup_test_server_with_jobs(vec![job], assignments).await;

    let response: TestResponse = test_server
        .server
        .get(&format!("/api/v1/schedules/{}", job_id))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    // Verify the response structure
    assert!(body["data"]["schedule_id"].is_string());
    assert!(body["data"]["assignments"].is_array());

    let assignments_array = body["data"]["assignments"].as_array().unwrap();
    if !assignments_array.is_empty() {
        let first_assignment = &assignments_array[0];
        assert!(first_assignment["staff_id"].is_string());
        assert!(first_assignment["date"].is_string());
        assert!(first_assignment["shift"].is_string());
    }
}

// ============================================================================
// Job Processing Tests with Mocked Data Service
// ============================================================================

use common::{create_sample_staff_list, MockDataServiceClient};
use scheduling_service::domain::schedule_generator::ScheduleGenerator;
use scheduling_service::infrastructure::JobProcessor;

/// Create a ScheduleGenerator with default rules for testing
fn create_test_scheduler() -> ScheduleGenerator {
    let rules: Vec<Arc<dyn scheduling_service::domain::rules::Rule>> = vec![
        Arc::new(MinDaysOffRule::new(1)),
        Arc::new(MaxDaysOffRule::new(2)),
        Arc::new(NoMorningAfterEveningRule::new()),
        Arc::new(ShiftBalanceRule::new(2)),
    ];
    ScheduleGenerator::new(rules)
}

/// Test job processing with successful data service response
#[tokio::test]
async fn test_job_processor_success_with_mock_data_service() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();

    // Create initial pending job
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    // Create mock repositories with the job pre-created
    let job_repo = Arc::new(MockScheduleJobRepository::with_jobs(vec![job]));
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::new());

    // Mock data service to return staff members (using mockall)
    let staff_list = create_sample_staff_list(3);
    let mut mock_client = MockDataServiceClient::new();
    mock_client
        .expect_get_group_members()
        .with(mockall::predicate::eq(group_id))
        .times(1)
        .returning(move |_| Ok(staff_list.clone()));

    let scheduler = Arc::new(create_test_scheduler());
    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        assignment_repo.clone(),
        Arc::new(mock_client),
        scheduler,
    ));

    // Start processor and get sender
    let (sender, _handle) = processor.start();

    // Send job request
    let request = scheduling_service::api::requests::schedule_request::ScheduleJobRequest {
        job_id,
        staff_group_id: group_id,
        period_begin_date: monday,
    };
    sender.send(request).await.unwrap();

    // Wait for processing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify job is completed
    let updated_job = job_repo.find_by_id(job_id).await.unwrap();
    assert!(updated_job.is_some());
    assert_eq!(updated_job.unwrap().status, JobStatus::Completed);

    // Verify assignments were created
    let assignments = assignment_repo.find_by_job_id(job_id).await.unwrap();
    assert!(!assignments.is_empty());
}

/// Test job processing when data service returns empty staff list
#[tokio::test]
async fn test_job_processor_empty_group_with_mock_data_service() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();

    // Create initial pending job
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    // Create mock repositories with the job pre-created
    let job_repo = Arc::new(MockScheduleJobRepository::with_jobs(vec![job]));
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::new());

    // Mock data service to return empty staff list (using mockall)
    let mut mock_client = MockDataServiceClient::new();
    mock_client
        .expect_get_group_members()
        .with(mockall::predicate::eq(group_id))
        .times(1)
        .returning(|_| Ok(vec![]));

    let scheduler = Arc::new(create_test_scheduler());
    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        assignment_repo.clone(),
        Arc::new(mock_client),
        scheduler,
    ));

    // Start processor and get sender
    let (sender, _handle) = processor.start();

    // Send job request
    let request = scheduling_service::api::requests::schedule_request::ScheduleJobRequest {
        job_id,
        staff_group_id: group_id,
        period_begin_date: monday,
    };
    sender.send(request).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify job failed due to empty group
    let updated_job = job_repo.find_by_id(job_id).await.unwrap();
    assert!(updated_job.is_some());
    assert_eq!(updated_job.unwrap().status, JobStatus::Failed);
}

/// Test job processing when data service returns an error
#[tokio::test]
async fn test_job_processor_data_service_error_with_mock() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();

    // Create initial pending job
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    // Create mock repositories with the job pre-created
    let job_repo = Arc::new(MockScheduleJobRepository::with_jobs(vec![job]));
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::new());

    // Mock data service to return an error (using mockall)
    let mut mock_client = MockDataServiceClient::new();
    mock_client
        .expect_get_group_members()
        .with(mockall::predicate::eq(group_id))
        .times(1)
        .returning(|_| {
            Err(shared::DomainError::ExternalServiceError(
                "Data service unavailable".to_string(),
            ))
        });

    let scheduler = Arc::new(create_test_scheduler());
    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        assignment_repo.clone(),
        Arc::new(mock_client),
        scheduler,
    ));

    // Start processor and get sender
    let (sender, _handle) = processor.start();

    // Send job request
    let request = scheduling_service::api::requests::schedule_request::ScheduleJobRequest {
        job_id,
        staff_group_id: group_id,
        period_begin_date: monday,
    };
    sender.send(request).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify job failed due to data service error
    let updated_job = job_repo.find_by_id(job_id).await.unwrap();
    assert!(updated_job.is_some());
    assert_eq!(updated_job.unwrap().status, JobStatus::Failed);
}

/// Test job processing with data service returning group not found
#[tokio::test]
async fn test_job_processor_group_not_found_with_mock_data_service() {
    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let monday = get_test_monday();

    // Create initial pending job
    let job = create_sample_job(job_id, group_id, monday, JobStatus::Pending);

    // Create mock repositories with the job pre-created
    let job_repo = Arc::new(MockScheduleJobRepository::with_jobs(vec![job]));
    let assignment_repo = Arc::new(MockShiftAssignmentRepository::new());

    // Mock data service to return NotFound error (using mockall)
    let mut mock_client = MockDataServiceClient::new();
    mock_client
        .expect_get_group_members()
        .with(mockall::predicate::eq(group_id))
        .times(1)
        .returning(|id| {
            Err(shared::DomainError::NotFound(format!(
                "Group {} not found",
                id
            )))
        });

    let scheduler = Arc::new(create_test_scheduler());
    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        assignment_repo.clone(),
        Arc::new(mock_client),
        scheduler,
    ));

    // Start processor and get sender
    let (sender, _handle) = processor.start();

    // Send job request
    let request = scheduling_service::api::requests::schedule_request::ScheduleJobRequest {
        job_id,
        staff_group_id: group_id,
        period_begin_date: monday,
    };
    sender.send(request).await.unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify job failed
    let updated_job = job_repo.find_by_id(job_id).await.unwrap();
    assert!(updated_job.is_some());
    assert_eq!(updated_job.unwrap().status, JobStatus::Failed);
}

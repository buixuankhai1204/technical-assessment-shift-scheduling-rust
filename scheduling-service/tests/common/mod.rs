use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use mockall::mock;
use scheduling_service::api::requests::schedule_request::ScheduleJobRequest;
use scheduling_service::api::AppState;
use scheduling_service::domain::entities::{ScheduleJob, ShiftAssignment};
use scheduling_service::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};
use scheduling_service::infrastructure::http_client::{DataServiceClientTrait, StaffResponse};
use shared::{DomainError, DomainResult, JobStatus, ShiftType, StaffStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;

// Generate mock for DataServiceClientTrait using mockall (for HTTP calls to data-service)
mock! {
    pub DataServiceClient {}

    #[async_trait]
    impl DataServiceClientTrait for DataServiceClient {
        async fn get_group_members(&self, group_id: Uuid) -> DomainResult<Vec<StaffResponse>>;
    }
}

/// Manual mock implementation for ScheduleJobRepository
#[derive(Default)]
pub struct MockScheduleJobRepository {
    jobs: RwLock<HashMap<Uuid, ScheduleJob>>,
}

impl MockScheduleJobRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_jobs(job_list: Vec<ScheduleJob>) -> Self {
        let repo = Self::new();
        {
            let mut jobs = repo.jobs.write().unwrap();
            for job in job_list {
                jobs.insert(job.id, job);
            }
        }
        repo
    }
}

#[async_trait]
impl ScheduleJobRepository for MockScheduleJobRepository {
    async fn create(&self, job: ScheduleJob) -> DomainResult<ScheduleJob> {
        self.jobs.write().unwrap().insert(job.id, job.clone());
        Ok(job)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScheduleJob>> {
        Ok(self.jobs.read().unwrap().get(&id).cloned())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(&id) {
            job.status = status;
            job.error_message = error_message;
            job.updated_at = Utc::now();
            Ok(())
        } else {
            Err(DomainError::NotFound(format!("Job {} not found", id)))
        }
    }

    async fn mark_completed(&self, id: Uuid) -> DomainResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(&id) {
            job.status = JobStatus::Completed;
            job.completed_at = Some(Utc::now());
            job.updated_at = Utc::now();
            Ok(())
        } else {
            Err(DomainError::NotFound(format!("Job {} not found", id)))
        }
    }

    async fn mark_failed(&self, id: Uuid, error_message: String) -> DomainResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(&id) {
            job.status = JobStatus::Failed;
            job.error_message = Some(error_message);
            job.updated_at = Utc::now();
            Ok(())
        } else {
            Err(DomainError::NotFound(format!("Job {} not found", id)))
        }
    }
}

/// Manual mock implementation for ShiftAssignmentRepository
#[derive(Default)]
pub struct MockShiftAssignmentRepository {
    assignments: RwLock<Vec<ShiftAssignment>>,
}

impl MockShiftAssignmentRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_assignments(assignment_list: Vec<ShiftAssignment>) -> Self {
        let repo = Self::new();
        {
            let mut assignments = repo.assignments.write().unwrap();
            *assignments = assignment_list;
        }
        repo
    }
}

#[async_trait]
impl ShiftAssignmentRepository for MockShiftAssignmentRepository {
    async fn create_batch(&self, assignments: Vec<ShiftAssignment>) -> DomainResult<()> {
        let mut current = self.assignments.write().unwrap();
        current.extend(assignments);
        Ok(())
    }

    async fn find_by_job_id(&self, job_id: Uuid) -> DomainResult<Vec<ShiftAssignment>> {
        let assignments = self.assignments.read().unwrap();
        Ok(assignments
            .iter()
            .filter(|a| a.schedule_job_id == job_id)
            .cloned()
            .collect())
    }
}

/// Create a sample staff response for testing
pub fn create_sample_staff_response(
    id: Uuid,
    name: &str,
    email: &str,
    position: &str,
) -> StaffResponse {
    let now = Utc::now();
    StaffResponse {
        id,
        name: name.to_string(),
        email: email.to_string(),
        position: position.to_string(),
        status: StaffStatus::Active,
        created_at: now,
        updated_at: now,
    }
}

/// Create multiple sample staff responses for testing
pub fn create_sample_staff_list(count: usize) -> Vec<StaffResponse> {
    (0..count)
        .map(|i| {
            create_sample_staff_response(
                Uuid::new_v4(),
                &format!("Staff {}", i + 1),
                &format!("staff{}@example.com", i + 1),
                "Employee",
            )
        })
        .collect()
}

/// Create test app state with mock repositories and a dummy channel
pub fn create_test_app_state(
    job_repo: Arc<dyn ScheduleJobRepository>,
    assignment_repo: Arc<dyn ShiftAssignmentRepository>,
) -> (AppState, mpsc::Receiver<ScheduleJobRequest>) {
    // Create a channel for job processing (with larger buffer for tests)
    let (sender, receiver) = mpsc::channel::<ScheduleJobRequest>(100);

    let state = AppState::new(job_repo, assignment_repo, sender);
    (state, receiver)
}

/// Struct to hold test server and keep receiver alive
pub struct TestServerWithReceiver {
    pub server: axum_test::TestServer,
    #[allow(dead_code)]
    pub receiver: mpsc::Receiver<ScheduleJobRequest>,
}

/// Create a sample schedule job for testing
pub fn create_sample_job(
    id: Uuid,
    staff_group_id: Uuid,
    period_begin_date: NaiveDate,
    status: JobStatus,
) -> ScheduleJob {
    let now = Utc::now();
    ScheduleJob {
        id,
        staff_group_id,
        period_begin_date,
        status,
        error_message: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
    }
}

/// Create a sample completed job with assignments
pub fn create_completed_job(
    id: Uuid,
    staff_group_id: Uuid,
    period_begin_date: NaiveDate,
) -> ScheduleJob {
    let now = Utc::now();
    ScheduleJob {
        id,
        staff_group_id,
        period_begin_date,
        status: JobStatus::Completed,
        error_message: None,
        created_at: now,
        updated_at: now,
        completed_at: Some(now),
    }
}

/// Create sample shift assignments for testing
pub fn create_sample_assignments(
    job_id: Uuid,
    staff_ids: Vec<Uuid>,
    start_date: NaiveDate,
) -> Vec<ShiftAssignment> {
    let now = Utc::now();
    let shifts = [ShiftType::Morning, ShiftType::Evening, ShiftType::DayOff];
    let mut assignments = Vec::new();

    for (day_offset, staff_id) in staff_ids.iter().enumerate() {
        for day in 0..7 {
            let date = start_date + chrono::Duration::days(day);
            let shift = shifts[(day_offset + day as usize) % 3];
            assignments.push(ShiftAssignment {
                id: Uuid::new_v4(),
                schedule_job_id: job_id,
                staff_id: *staff_id,
                date,
                shift,
                created_at: now,
            });
        }
    }

    assignments
}

/// Get a Monday date for testing (schedules must start on Monday)
pub fn get_test_monday() -> NaiveDate {
    // Use a fixed Monday for consistent tests
    NaiveDate::from_ymd_opt(2026, 2, 9).unwrap() // A Monday
}

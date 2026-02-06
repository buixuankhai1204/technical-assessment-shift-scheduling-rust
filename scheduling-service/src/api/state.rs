use std::sync::Arc;
use tokio::sync::mpsc;
use crate::api::requests::schedule_request::ScheduleJobRequest;
use crate::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};

#[derive(Clone)]
pub struct AppState {
    pub job_repo: Arc<dyn ScheduleJobRepository>,
    pub assignment_repo: Arc<dyn ShiftAssignmentRepository>,
    pub schedule_sender: mpsc::Sender<ScheduleJobRequest>,
}

impl AppState {
    pub fn new(
        job_repo: Arc<dyn ScheduleJobRepository>,
        assignment_repo: Arc<dyn ShiftAssignmentRepository>,
        schedule_sender: mpsc::Sender<ScheduleJobRequest>,
    ) -> Self {
        Self {
            job_repo,
            assignment_repo,
            schedule_sender,
        }
    }
}

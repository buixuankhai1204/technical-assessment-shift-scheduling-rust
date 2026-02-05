use chrono::{NaiveDate, Utc};
use shared::{DomainError, DomainResult, JobStatus};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::domain::entities::ShiftAssignment;
use crate::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};
use crate::domain::services::{GreedyScheduler, SchedulingRules};
use crate::infrastructure::http_client::DataServiceClient;

/// Message sent to the scheduler processor
#[derive(Debug)]
pub struct ScheduleRequest {
    pub job_id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

/// Schedule processor service that handles async job processing
pub struct ScheduleProcessor {
    job_repo: Arc<dyn ScheduleJobRepository>,
    assignment_repo: Arc<dyn ShiftAssignmentRepository>,
    data_service_client: Arc<DataServiceClient>,
    scheduler: GreedyScheduler,
}

impl ScheduleProcessor {
    pub fn new(
        job_repo: Arc<dyn ScheduleJobRepository>,
        assignment_repo: Arc<dyn ShiftAssignmentRepository>,
        data_service_client: Arc<DataServiceClient>,
        rules: SchedulingRules,
    ) -> Self {
        Self {
            job_repo,
            assignment_repo,
            data_service_client,
            scheduler: GreedyScheduler::new(rules),
        }
    }

    /// Start the background processor
    pub fn start(self: Arc<Self>) -> (mpsc::Sender<ScheduleRequest>, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<ScheduleRequest>(100);

        let handle = tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let Err(e) = self.process_schedule_request(request).await {
                    tracing::error!("Failed to process schedule request: {:?}", e);
                }
            }
        });

        (tx, handle)
    }

    /// Process a single schedule request
    async fn process_schedule_request(&self, request: ScheduleRequest) -> DomainResult<()> {
        tracing::info!("Processing schedule request for job {}", request.job_id);

        // Update job status to processing
        self.job_repo
            .update_status(request.job_id, JobStatus::Processing, None)
            .await?;

        // Execute the scheduling logic
        match self.execute_scheduling(&request).await {
            Ok(()) => {
                self.job_repo.mark_completed(request.job_id).await?;
                tracing::info!("Successfully completed job {}", request.job_id);
                Ok(())
            }
            Err(e) => {
                let error_message = format!("Scheduling failed: {:?}", e);
                self.job_repo
                    .mark_failed(request.job_id, error_message.clone())
                    .await?;
                tracing::error!("Job {} failed: {}", request.job_id, error_message);
                Err(e)
            }
        }
    }

    /// Execute the actual scheduling logic
    async fn execute_scheduling(&self, request: &ScheduleRequest) -> DomainResult<()> {
        // Fetch staff members from the data service
        let staff_members = self
            .data_service_client
            .get_group_members(request.staff_group_id)
            .await
            .map_err(|e| DomainError::ExternalServiceError(e.to_string()))?;

        if staff_members.is_empty() {
            return Err(DomainError::InvalidInput(
                "No active staff members found in the group".to_string(),
            ));
        }

        let staff_ids: Vec<Uuid> = staff_members.iter().map(|s| s.id).collect();

        tracing::info!(
            "Generating schedule for {} staff members starting {}",
            staff_ids.len(),
            request.period_begin_date
        );

        // Generate the schedule using the greedy algorithm
        let schedule_state = self
            .scheduler
            .generate_schedule(staff_ids, request.period_begin_date)?;

        // Convert schedule state to shift assignments
        let assignments: Vec<ShiftAssignment> = schedule_state
            .get_all_assignments()
            .into_iter()
            .map(|(staff_id, date, shift)| ShiftAssignment {
                id: Uuid::new_v4(),
                schedule_job_id: request.job_id,
                staff_id,
                date,
                shift,
                created_at: Utc::now(),
            })
            .collect();

        tracing::info!("Generated {} shift assignments", assignments.len());

        // Save assignments to database
        self.assignment_repo.create_batch(assignments).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};
    use async_trait::async_trait;
    use shared::JobStatus;
    use std::sync::Mutex;
    use crate::domain::entities::ScheduleJob;

    struct MockJobRepository {
        updates: Arc<Mutex<Vec<(Uuid, JobStatus)>>>,
    }

    #[async_trait]
    impl ScheduleJobRepository for MockJobRepository {
        async fn create(&self, job: ScheduleJob) -> DomainResult<ScheduleJob> {
            Ok(job)
        }

        async fn find_by_id(&self, _id: Uuid) -> DomainResult<Option<ScheduleJob>> {
            Ok(None)
        }

        async fn update_status(
            &self,
            id: Uuid,
            status: JobStatus,
            _error_message: Option<String>,
        ) -> DomainResult<()> {
            self.updates.lock().unwrap().push((id, status));
            Ok(())
        }

        async fn mark_completed(&self, id: Uuid) -> DomainResult<()> {
            self.updates
                .lock()
                .unwrap()
                .push((id, JobStatus::Completed));
            Ok(())
        }

        async fn mark_failed(&self, id: Uuid, _error_message: String) -> DomainResult<()> {
            self.updates.lock().unwrap().push((id, JobStatus::Failed));
            Ok(())
        }
    }

    struct MockAssignmentRepository;

    #[async_trait]
    impl ShiftAssignmentRepository for MockAssignmentRepository {
        async fn create_batch(&self, _assignments: Vec<ShiftAssignment>) -> DomainResult<()> {
            Ok(())
        }

        async fn find_by_job_id(&self, _job_id: Uuid) -> DomainResult<Vec<ShiftAssignment>> {
            Ok(Vec::new())
        }

        async fn delete_by_job_id(&self, _job_id: Uuid) -> DomainResult<()> {
            Ok(())
        }
    }
}

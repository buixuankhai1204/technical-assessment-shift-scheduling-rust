use shared::{DomainError, DomainResult, JobStatus};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::api::requests::schedule_request::ScheduleJobRequest;
use crate::domain::repositories::{ScheduleJobRepository, ShiftAssignmentRepository};
use crate::domain::schedule_generator::ScheduleGenerator;
use crate::infrastructure::http_client::DataServiceClientTrait;

pub struct JobProcessor {
    job_repo: Arc<dyn ScheduleJobRepository>,
    assignment_repo: Arc<dyn ShiftAssignmentRepository>,
    data_service_client: Arc<dyn DataServiceClientTrait>,
    scheduler: Arc<ScheduleGenerator>,
}

impl JobProcessor {
    pub fn new(
        job_repo: Arc<dyn ScheduleJobRepository>,
        assignment_repo: Arc<dyn ShiftAssignmentRepository>,
        data_service_client: Arc<dyn DataServiceClientTrait>,
        scheduler: Arc<ScheduleGenerator>,
    ) -> Self {
        Self {
            job_repo,
            assignment_repo,
            data_service_client,
            scheduler,
        }
    }

    pub fn start(
        self: Arc<Self>,
    ) -> (
        mpsc::Sender<ScheduleJobRequest>,
        tokio::task::JoinHandle<()>,
    ) {
        let (tx, mut rx) = mpsc::channel::<ScheduleJobRequest>(100);

        let handle = tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let Err(e) = self.process_job(request).await {
                    tracing::error!("Failed to process schedule job: {:?}", e);
                }
            }
        });

        (tx, handle)
    }

    /// Process a single schedule job
    async fn process_job(&self, request: ScheduleJobRequest) -> DomainResult<()> {
        tracing::info!("Processing schedule job {}", request.job_id);

        self.job_repo
            .update_status(request.job_id, JobStatus::Processing, None)
            .await?;

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
    async fn execute_scheduling(&self, request: &ScheduleJobRequest) -> DomainResult<()> {
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

        // Generate the schedule
        let assignments = self.scheduler.generate_schedule(
            staff_ids,
            request.period_begin_date,
            request.job_id,
        )?;

        tracing::info!("Generated {} shift assignments", assignments.len());

        // Save assignments to database
        self.assignment_repo.create_batch(assignments).await?;

        Ok(())
    }
}

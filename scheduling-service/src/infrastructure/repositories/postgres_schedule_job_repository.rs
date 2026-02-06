use async_trait::async_trait;
use shared::{DomainError, DomainResult, JobStatus};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::ScheduleJob;
use crate::domain::repositories::ScheduleJobRepository;

pub struct PostgresScheduleJobRepository {
    pool: PgPool,
}

impl PostgresScheduleJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScheduleJobRepository for PostgresScheduleJobRepository {
    async fn create(&self, job: ScheduleJob) -> DomainResult<ScheduleJob> {
        let created_job = sqlx::query_as::<_, ScheduleJob>(
            r#"
            INSERT INTO schedule_jobs (id, staff_group_id, period_begin_date, status, error_message, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, staff_group_id, period_begin_date, status, error_message, created_at, updated_at, completed_at
            "#,
        )
        .bind(job.id)
        .bind(job.staff_group_id)
        .bind(job.period_begin_date)
        .bind(job.status)
        .bind(job.error_message)
        .bind(job.created_at)
        .bind(job.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(created_job)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScheduleJob>> {
        let job = sqlx::query_as::<_, ScheduleJob>(
            r#"
            SELECT id, staff_group_id, period_begin_date, status, error_message, created_at, updated_at, completed_at
            FROM schedule_jobs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(job)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE schedule_jobs
            SET status = $1, error_message = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_completed(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE schedule_jobs
            SET status = $1, completed_at = NOW(), updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(JobStatus::Completed)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error_message: String) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE schedule_jobs
            SET status = $1, error_message = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(JobStatus::Failed)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

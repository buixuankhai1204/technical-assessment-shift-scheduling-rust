use async_trait::async_trait;
use shared::{DomainError, DomainResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::ShiftAssignment;
use crate::domain::repositories::ShiftAssignmentRepository;

pub struct PostgresShiftAssignmentRepository {
    pool: PgPool,
}

impl PostgresShiftAssignmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShiftAssignmentRepository for PostgresShiftAssignmentRepository {
    async fn create_batch(&self, assignments: Vec<ShiftAssignment>) -> DomainResult<()> {
        if assignments.is_empty() {
            return Ok(());
        }

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        for assignment in assignments {
            sqlx::query(
                r#"
                INSERT INTO shift_assignments (id, schedule_job_id, staff_id, date, shift, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(assignment.id)
            .bind(assignment.schedule_job_id)
            .bind(assignment.staff_id)
            .bind(assignment.date)
            .bind(assignment.shift)
            .bind(assignment.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_job_id(&self, job_id: Uuid) -> DomainResult<Vec<ShiftAssignment>> {
        let assignments = sqlx::query_as::<_, ShiftAssignment>(
            r#"
            SELECT id, schedule_job_id, staff_id, date, shift, created_at
            FROM shift_assignments
            WHERE schedule_job_id = $1
            ORDER BY date, staff_id
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(assignments)
    }
}

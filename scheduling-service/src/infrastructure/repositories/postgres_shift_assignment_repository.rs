use async_trait::async_trait;
use shared::{DomainError, DomainResult};
use sqlx::{PgPool, Postgres, QueryBuilder};
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

        // Use batch insert with QueryBuilder for better performance
        // PostgreSQL has a limit on the number of bind parameters, so we chunk the inserts
        const BATCH_SIZE: usize = 1000;

        for chunk in assignments.chunks(BATCH_SIZE) {
            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                "INSERT INTO shift_assignments (id, schedule_job_id, staff_id, date, shift, created_at) "
            );

            query_builder.push_values(chunk, |mut b, assignment| {
                b.push_bind(assignment.id)
                    .push_bind(assignment.schedule_job_id)
                    .push_bind(assignment.staff_id)
                    .push_bind(assignment.date)
                    .push_bind(assignment.shift)
                    .push_bind(assignment.created_at);
            });

            query_builder
                .build()
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

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

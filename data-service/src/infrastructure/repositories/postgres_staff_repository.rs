use async_trait::async_trait;
use shared::{DomainError, DomainResult, PaginationParams, StaffStatus};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::requests::{CreateStaffRequest, UpdateStaffRequest};
use crate::domain::entities::Staff;
use crate::domain::repositories::StaffRepository;

pub struct PostgresStaffRepository {
    pool: PgPool,
}

impl PostgresStaffRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StaffRepository for PostgresStaffRepository {
    async fn create(&self, request: CreateStaffRequest) -> DomainResult<Staff> {
        let status = request.status.unwrap_or(StaffStatus::Active);

        let staff = sqlx::query_as::<_, Staff>(
            r#"
            INSERT INTO staff (name, email, position, status)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, position, status, created_at, updated_at
            "#,
        )
        .bind(&request.name)
        .bind(&request.email)
        .bind(&request.position)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(staff)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Staff>> {
        let staff = sqlx::query_as::<_, Staff>(
            r#"
            SELECT id, name, email, position, status, created_at, updated_at
            FROM staff
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(staff)
    }

    async fn find_by_email(&self, email: &str) -> DomainResult<Option<Staff>> {
        let staff = sqlx::query_as::<_, Staff>(
            r#"
            SELECT id, name, email, position, status, created_at, updated_at
            FROM staff
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(staff)
    }

    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<Staff>, u64)> {
        let offset = (params.page - 1) * params.page_size;

        let staff_list = sqlx::query_as::<_, Staff>(
            r#"
            SELECT id, name, email, position, status, created_at, updated_at
            FROM staff
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(params.page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM staff")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((staff_list, total.0 as u64))
    }

    async fn list_by_status(
        &self,
        status: StaffStatus,
        params: PaginationParams,
    ) -> DomainResult<(Vec<Staff>, u64)> {
        let offset = (params.page - 1) * params.page_size;

        let staff_list = sqlx::query_as::<_, Staff>(
            r#"
            SELECT id, name, email, position, status, created_at, updated_at
            FROM staff
            WHERE status = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&status)
        .bind(params.page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM staff WHERE status = $1")
            .bind(&status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((staff_list, total.0 as u64))
    }

    async fn update(&self, id: Uuid, request: UpdateStaffRequest) -> DomainResult<Staff> {
        // Fetch current staff
        let current = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Staff with id {} not found", id)))?;

        let staff = sqlx::query_as::<_, Staff>(
            r#"
            UPDATE staff
            SET name = $1, email = $2, position = $3, status = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, email, position, status, created_at, updated_at
            "#,
        )
        .bind(request.name.unwrap_or(current.name))
        .bind(request.email.unwrap_or(current.email))
        .bind(request.position.unwrap_or(current.position))
        .bind(request.status.unwrap_or(current.status))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(staff)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query("DELETE FROM staff WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "Staff with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn find_by_group_id(&self, group_id: Uuid) -> DomainResult<Vec<Staff>> {
        let staff_list = sqlx::query_as::<_, Staff>(
            r#"
            SELECT s.id, s.name, s.email, s.position, s.status, s.created_at, s.updated_at
            FROM staff s
            INNER JOIN group_memberships gm ON s.id = gm.staff_id
            WHERE gm.group_id = $1
            ORDER BY s.name
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(staff_list)
    }
}

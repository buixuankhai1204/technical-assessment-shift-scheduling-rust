use async_trait::async_trait;
use shared::{DomainError, DomainResult, PaginationParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{CreateGroupRequest, StaffGroup, UpdateGroupRequest};
use crate::domain::repositories::GroupRepository;

pub struct PostgresGroupRepository {
    pool: PgPool,
}

impl PostgresGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for PostgresGroupRepository {
    async fn create(&self, request: CreateGroupRequest) -> DomainResult<StaffGroup> {
        let group = sqlx::query_as::<_, StaffGroup>(
            r#"
            INSERT INTO staff_groups (name, parent_id)
            VALUES ($1, $2)
            RETURNING id, name, parent_id, created_at, updated_at
            "#,
        )
        .bind(&request.name)
        .bind(request.parent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<StaffGroup>> {
        let group = sqlx::query_as::<_, StaffGroup>(
            r#"
            SELECT id, name, parent_id, created_at, updated_at
            FROM staff_groups
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<StaffGroup>, u64)> {
        let offset = (params.page - 1) * params.page_size;

        let groups = sqlx::query_as::<_, StaffGroup>(
            r#"
            SELECT id, name, parent_id, created_at, updated_at
            FROM staff_groups
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(params.page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM staff_groups")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((groups, total.0 as u64))
    }

    async fn list_by_parent_id(&self, parent_id: Uuid) -> DomainResult<Vec<StaffGroup>> {
        let groups = sqlx::query_as::<_, StaffGroup>(
            r#"
            SELECT id, name, parent_id, created_at, updated_at
            FROM staff_groups
            WHERE parent_id = $1
            ORDER BY name
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(groups)
    }

    async fn update(&self, id: Uuid, request: UpdateGroupRequest) -> DomainResult<StaffGroup> {
        let current = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Group with id {} not found", id)))?;

        let group = sqlx::query_as::<_, StaffGroup>(
            r#"
            UPDATE staff_groups
            SET name = $1, parent_id = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, parent_id, created_at, updated_at
            "#,
        )
        .bind(request.name.unwrap_or(current.name))
        .bind(request.parent_id.or(current.parent_id))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query("DELETE FROM staff_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "Group with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_descendant_ids(&self, group_id: Uuid) -> DomainResult<Vec<Uuid>> {
        // Recursive CTE to get all descendants
        let descendants = sqlx::query_scalar::<_, Uuid>(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM staff_groups WHERE id = $1
                UNION
                SELECT sg.id FROM staff_groups sg
                INNER JOIN descendants d ON sg.parent_id = d.id
            )
            SELECT id FROM descendants WHERE id != $1
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(descendants)
    }
}

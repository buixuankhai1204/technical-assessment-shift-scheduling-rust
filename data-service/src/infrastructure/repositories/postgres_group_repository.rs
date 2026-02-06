use async_trait::async_trait;
use shared::{DomainError, DomainResult, PaginationParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::requests::{CreateGroupRequest, UpdateGroupRequest};
use crate::domain::entities::{GroupWithMembers, Staff, StaffGroup};
use crate::domain::repositories::GroupRepository;

#[derive(sqlx::FromRow)]
struct ResolvedMemberRow {
    group_id: Uuid,
    group_name: String,
    group_parent_id: Option<Uuid>,
    group_created_at: chrono::DateTime<chrono::Utc>,
    group_updated_at: chrono::DateTime<chrono::Utc>,
    staff_id: Uuid,
    staff_name: String,
    staff_email: String,
    staff_position: String,
    staff_status: shared::StaffStatus,
    staff_created_at: chrono::DateTime<chrono::Utc>,
    staff_updated_at: chrono::DateTime<chrono::Utc>,
}

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

    async fn find_by_name(&self, name: &str) -> DomainResult<Option<StaffGroup>> {
        let group = sqlx::query_as::<_, StaffGroup>(
            r#"
            SELECT id, name, parent_id, created_at, updated_at
            FROM staff_groups
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    async fn get_resolved_members(
        &self,
        group_id: Uuid,
    ) -> DomainResult<(Vec<GroupWithMembers>, u64)> {
        let rows = sqlx::query_as::<_, ResolvedMemberRow>(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM staff_groups WHERE id = $1
                UNION
                SELECT sg.id FROM staff_groups sg
                INNER JOIN descendants d ON sg.parent_id = d.id
            )
            SELECT
                sg.id          AS group_id,
                sg.name        AS group_name,
                sg.parent_id   AS group_parent_id,
                sg.created_at  AS group_created_at,
                sg.updated_at  AS group_updated_at,
                s.id           AS staff_id,
                s.name         AS staff_name,
                s.email        AS staff_email,
                s.position     AS staff_position,
                s.status       AS staff_status,
                s.created_at   AS staff_created_at,
                s.updated_at   AS staff_updated_at
            FROM descendants d
            JOIN staff_groups sg       ON sg.id = d.id
            JOIN group_memberships gm  ON gm.group_id = sg.id
            JOIN staff s               ON s.id = gm.staff_id
            WHERE s.status = 'ACTIVE'
            ORDER BY sg.name, s.name
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let unique_count = {
            let mut ids: Vec<Uuid> = rows.iter().map(|r| r.staff_id).collect();
            ids.sort();
            ids.dedup();
            ids.len() as u64
        };

        let mut result: Vec<GroupWithMembers> = Vec::new();
        let mut current_group_id: Option<Uuid> = None;

        for row in rows {
            let staff = Staff {
                id: row.staff_id,
                name: row.staff_name,
                email: row.staff_email,
                position: row.staff_position,
                status: row.staff_status,
                created_at: row.staff_created_at,
                updated_at: row.staff_updated_at,
            };

            if current_group_id == Some(row.group_id) {
                // Same group — push member to the last entry
                result.last_mut().unwrap().members.push(staff);
            } else {
                current_group_id = Some(row.group_id);
                let group = StaffGroup {
                    id: row.group_id,
                    name: row.group_name,
                    parent_id: row.group_parent_id,
                    created_at: row.group_created_at,
                    updated_at: row.group_updated_at,
                };
                result.push(GroupWithMembers {
                    group,
                    members: vec![staff],
                });
            }
        }

        Ok((result, unique_count))
    }
}

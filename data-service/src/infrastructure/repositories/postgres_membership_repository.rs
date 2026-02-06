use async_trait::async_trait;
use shared::{DomainError, DomainResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::GroupMembership;
use crate::domain::repositories::MembershipRepository;

pub struct PostgresMembershipRepository {
    pool: PgPool,
}

impl PostgresMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MembershipRepository for PostgresMembershipRepository {
    async fn add_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<GroupMembership> {
        let membership = sqlx::query_as::<_, GroupMembership>(
            r#"
            INSERT INTO group_memberships (staff_id, group_id)
            VALUES ($1, $2)
            ON CONFLICT (staff_id, group_id) DO NOTHING
            RETURNING id, staff_id, group_id, created_at
            "#,
        )
        .bind(staff_id)
        .bind(group_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(membership)
    }

    async fn remove_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<()> {
        let result =
            sqlx::query("DELETE FROM group_memberships WHERE staff_id = $1 AND group_id = $2")
                .bind(staff_id)
                .bind(group_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound("Membership not found".to_string()));
        }

        Ok(())
    }
}

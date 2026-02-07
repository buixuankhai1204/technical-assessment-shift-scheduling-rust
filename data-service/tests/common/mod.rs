use async_trait::async_trait;
use chrono::Utc;
use data_service::api::AppState;
use data_service::api::requests::{CreateGroupRequest, CreateStaffRequest, UpdateGroupRequest, UpdateStaffRequest};
use data_service::domain::entities::{GroupMembership, GroupWithMembers, Staff, StaffGroup};
use data_service::domain::repositories::{GroupRepository, MembershipRepository, StaffRepository};
use data_service::infrastructure::redis::RedisPool;
use shared::{DomainError, DomainResult, PaginationParams, StaffStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Default)]
pub struct MockStaffRepository {
    staff: RwLock<HashMap<Uuid, Staff>>,
}

impl MockStaffRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_staff(staff_list: Vec<Staff>) -> Self {
        let repo = Self::new();
        {
            let mut staff = repo.staff.write().unwrap();
            for s in staff_list {
                staff.insert(s.id, s);
            }
        }
        repo
    }
}

#[async_trait]
impl StaffRepository for MockStaffRepository {
    async fn create(&self, request: CreateStaffRequest) -> DomainResult<Staff> {
        let now = Utc::now();
        let staff = Staff {
            id: Uuid::new_v4(),
            name: request.name,
            email: request.email,
            position: request.position,
            status: request.status.unwrap_or(StaffStatus::Active),
            created_at: now,
            updated_at: now,
        };
        self.staff.write().unwrap().insert(staff.id, staff.clone());
        Ok(staff)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Staff>> {
        Ok(self.staff.read().unwrap().get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> DomainResult<Option<Staff>> {
        Ok(self
            .staff
            .read()
            .unwrap()
            .values()
            .find(|s| s.email == email)
            .cloned())
    }

    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<Staff>, u64)> {
        let staff = self.staff.read().unwrap();
        let all: Vec<Staff> = staff.values().cloned().collect();
        let total = all.len() as u64;
        let offset = (params.page - 1) * params.page_size;
        let paginated: Vec<Staff> = all
            .into_iter()
            .skip(offset as usize)
            .take(params.page_size as usize)
            .collect();
        Ok((paginated, total))
    }

    async fn list_by_status(
        &self,
        status: StaffStatus,
        params: PaginationParams,
    ) -> DomainResult<(Vec<Staff>, u64)> {
        let staff = self.staff.read().unwrap();
        let filtered: Vec<Staff> = staff
            .values()
            .filter(|s| s.status == status)
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let offset = (params.page - 1) * params.page_size;
        let paginated: Vec<Staff> = filtered
            .into_iter()
            .skip(offset as usize)
            .take(params.page_size as usize)
            .collect();
        Ok((paginated, total))
    }

    async fn update(&self, id: Uuid, request: UpdateStaffRequest) -> DomainResult<Staff> {
        let mut staff_map = self.staff.write().unwrap();
        let staff = staff_map
            .get_mut(&id)
            .ok_or_else(|| DomainError::NotFound(format!("Staff with id {} not found", id)))?;

        if let Some(name) = request.name {
            staff.name = name;
        }
        if let Some(email) = request.email {
            staff.email = email;
        }
        if let Some(position) = request.position {
            staff.position = position;
        }
        if let Some(status) = request.status {
            staff.status = status;
        }
        staff.updated_at = Utc::now();

        Ok(staff.clone())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        self.staff
            .write()
            .unwrap()
            .remove(&id)
            .ok_or_else(|| DomainError::NotFound(format!("Staff with id {} not found", id)))?;
        Ok(())
    }

    async fn find_by_group_id(&self, _group_id: Uuid) -> DomainResult<Vec<Staff>> {
        // Simplified: return all active staff for testing
        let staff = self.staff.read().unwrap();
        Ok(staff
            .values()
            .filter(|s| s.status == StaffStatus::Active)
            .cloned()
            .collect())
    }
}

/// Mock Group Repository for testing
#[derive(Default)]
pub struct MockGroupRepository {
    groups: RwLock<HashMap<Uuid, StaffGroup>>,
}

impl MockGroupRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_groups(group_list: Vec<StaffGroup>) -> Self {
        let repo = Self::new();
        {
            let mut groups = repo.groups.write().unwrap();
            for g in group_list {
                groups.insert(g.id, g);
            }
        }
        repo
    }
}

#[async_trait]
impl GroupRepository for MockGroupRepository {
    async fn create(&self, request: CreateGroupRequest) -> DomainResult<StaffGroup> {
        let now = Utc::now();
        let group = StaffGroup {
            id: Uuid::new_v4(),
            name: request.name,
            parent_id: request.parent_id,
            created_at: now,
            updated_at: now,
        };
        self.groups.write().unwrap().insert(group.id, group.clone());
        Ok(group)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<StaffGroup>> {
        Ok(self.groups.read().unwrap().get(&id).cloned())
    }

    async fn list(&self, params: PaginationParams) -> DomainResult<(Vec<StaffGroup>, u64)> {
        let groups = self.groups.read().unwrap();
        let all: Vec<StaffGroup> = groups.values().cloned().collect();
        let total = all.len() as u64;
        let offset = (params.page - 1) * params.page_size;
        let paginated: Vec<StaffGroup> = all
            .into_iter()
            .skip(offset as usize)
            .take(params.page_size as usize)
            .collect();
        Ok((paginated, total))
    }

    async fn update(&self, id: Uuid, request: UpdateGroupRequest) -> DomainResult<StaffGroup> {
        let mut groups_map = self.groups.write().unwrap();
        let group = groups_map
            .get_mut(&id)
            .ok_or_else(|| DomainError::NotFound(format!("Group with id {} not found", id)))?;

        if let Some(name) = request.name {
            group.name = name;
        }
        if let Some(parent_id) = request.parent_id {
            group.parent_id = Some(parent_id);
        }
        group.updated_at = Utc::now();

        Ok(group.clone())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        self.groups
            .write()
            .unwrap()
            .remove(&id)
            .ok_or_else(|| DomainError::NotFound(format!("Group with id {} not found", id)))?;
        Ok(())
    }

    async fn find_by_name(&self, name: &str) -> DomainResult<Option<StaffGroup>> {
        Ok(self
            .groups
            .read()
            .unwrap()
            .values()
            .find(|g| g.name == name)
            .cloned())
    }

    async fn get_resolved_members(
        &self,
        group_id: Uuid,
    ) -> DomainResult<(Vec<GroupWithMembers>, u64)> {
        let groups = self.groups.read().unwrap();
        if let Some(group) = groups.get(&group_id) {
            Ok((
                vec![GroupWithMembers {
                    group: group.clone(),
                    members: vec![],
                }],
                0,
            ))
        } else {
            Err(DomainError::NotFound(format!(
                "Group with id {} not found",
                group_id
            )))
        }
    }
}

/// Mock Membership Repository for testing
#[derive(Default)]
pub struct MockMembershipRepository {
    memberships: RwLock<Vec<GroupMembership>>,
}

impl MockMembershipRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MembershipRepository for MockMembershipRepository {
    async fn add_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<GroupMembership> {
        let membership = GroupMembership {
            id: Uuid::new_v4(),
            staff_id,
            group_id,
            created_at: Utc::now(),
        };
        self.memberships.write().unwrap().push(membership.clone());
        Ok(membership)
    }

    async fn remove_member(&self, staff_id: Uuid, group_id: Uuid) -> DomainResult<()> {
        let mut memberships = self.memberships.write().unwrap();
        let initial_len = memberships.len();
        memberships.retain(|m| !(m.staff_id == staff_id && m.group_id == group_id));
        if memberships.len() == initial_len {
            Err(DomainError::NotFound(
                "Membership not found".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Mock Redis Pool for testing (no-op implementation)
pub async fn create_mock_redis_pool() -> RedisPool {
    // Create a dummy redis connection that we won't actually use
    // In tests, we'll use a real Redis or skip cache-related functionality
    let client = redis::Client::open("redis://localhost:6379").unwrap();
    redis::aio::ConnectionManager::new(client).await.unwrap()
}

/// Create test app state with mock repositories
pub fn create_test_app_state(
    staff_repo: Arc<dyn StaffRepository>,
    group_repo: Arc<dyn GroupRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
    redis_pool: RedisPool,
) -> AppState {
    AppState::new(staff_repo, group_repo, membership_repo, redis_pool)
}

/// Create a sample staff for testing
pub fn create_sample_staff(id: Uuid, name: &str, email: &str) -> Staff {
    let now = Utc::now();
    Staff {
        id,
        name: name.to_string(),
        email: email.to_string(),
        position: "Developer".to_string(),
        status: StaffStatus::Active,
        created_at: now,
        updated_at: now,
    }
}

/// Create a sample group for testing
pub fn create_sample_group(id: Uuid, name: &str, parent_id: Option<Uuid>) -> StaffGroup {
    let now = Utc::now();
    StaffGroup {
        id,
        name: name.to_string(),
        parent_id,
        created_at: now,
        updated_at: now,
    }
}


use std::sync::Arc;

use crate::domain::repositories::{GroupRepository, MembershipRepository, StaffRepository};
use crate::infrastructure::redis::RedisPool;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub staff_repo: Arc<dyn StaffRepository>,
    pub group_repo: Arc<dyn GroupRepository>,
    pub membership_repo: Arc<dyn MembershipRepository>,
    pub redis_pool: RedisPool,
}

impl AppState {
    pub fn new(
        staff_repo: Arc<dyn StaffRepository>,
        group_repo: Arc<dyn GroupRepository>,
        membership_repo: Arc<dyn MembershipRepository>,
        redis_pool: RedisPool,
    ) -> Self {
        Self {
            staff_repo,
            group_repo,
            membership_repo,
            redis_pool,
        }
    }
}

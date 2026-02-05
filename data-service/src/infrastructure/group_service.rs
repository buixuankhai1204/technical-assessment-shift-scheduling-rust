use futures::future::try_join_all;
use shared::{DomainResult, StaffStatus};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::Staff;
use crate::domain::repositories::{GroupRepository, MembershipRepository, StaffRepository};

/// Service for group-related business logic
pub struct GroupService {
    group_repo: Arc<dyn GroupRepository>,
    staff_repo: Arc<dyn StaffRepository>,
    membership_repo: Arc<dyn MembershipRepository>,
}

impl GroupService {
    pub fn new(
        group_repo: Arc<dyn GroupRepository>,
        staff_repo: Arc<dyn StaffRepository>,
        membership_repo: Arc<dyn MembershipRepository>,
    ) -> Self {
        Self {
            group_repo,
            staff_repo,
            membership_repo,
        }
    }

    /// Get all staff members in a group, including members of all nested subgroups
    /// Only returns ACTIVE staff
    pub async fn get_resolved_members(&self, group_id: Uuid) -> DomainResult<Vec<Staff>> {
        // Get the group and all its descendants
        let mut group_ids = vec![group_id];
        group_ids.extend(self.group_repo.get_descendant_ids(group_id).await?);

        // Collect all unique staff IDs from all groups concurrently
        let membership_futures = group_ids
            .iter()
            .map(|gid| self.membership_repo.find_by_group_id(*gid));

        let membership_results = try_join_all(membership_futures).await?;

        // Collect unique staff IDs
        let staff_ids: Vec<Uuid> = membership_results
            .into_iter()
            .flatten()
            .map(|m| m.staff_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Fetch all staff in a single batch query
        let all_staff = self.staff_repo.find_by_ids(staff_ids).await?;

        // Filter by ACTIVE status and sort
        let mut active_staff: Vec<Staff> = all_staff
            .into_iter()
            .filter(|staff| staff.status == StaffStatus::Active)
            .collect();

        active_staff.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(active_staff)
    }
}

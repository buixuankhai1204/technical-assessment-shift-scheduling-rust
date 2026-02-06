pub mod group;
pub mod membership;
pub mod staff;

pub use group::StaffGroup;
pub use membership::GroupMembership;
pub use staff::Staff;

/// A subgroup together with its direct active members
pub struct GroupWithMembers {
    pub group: StaffGroup,
    pub members: Vec<Staff>,
}

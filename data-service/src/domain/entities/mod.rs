pub mod group;
pub mod membership;
pub mod staff;

pub use group::StaffGroup;
pub use membership::GroupMembership;
pub use staff::Staff;

pub struct GroupWithMembers {
    pub group: StaffGroup,
    pub members: Vec<Staff>,
}

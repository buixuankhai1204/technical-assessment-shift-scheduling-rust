pub mod max_days_off_rule;
pub mod min_days_off_rule;
pub mod no_morning_after_evening_rule;
pub mod shift_balance_rule;

use chrono::NaiveDate;
use shared::{DomainResult, ShiftType};
use std::collections::HashMap;
use uuid::Uuid;

pub use max_days_off_rule::MaxDaysOffRule;
pub use min_days_off_rule::MinDaysOffRule;
pub use no_morning_after_evening_rule::NoMorningAfterEveningRule;
pub use shift_balance_rule::ShiftBalanceRule;

#[derive(Debug, Clone)]
pub struct AssignmentContext {
    pub assignments: HashMap<Uuid, HashMap<NaiveDate, ShiftType>>,
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
}

pub trait Rule: Send + Sync {
    /// Check if the assignment violates this rule
    fn validate(&self, context: &AssignmentContext) -> DomainResult<()>;
}

use super::{AssignmentContext, Rule};
use shared::{DomainError, DomainResult, ShiftType};
use std::collections::HashMap;

pub struct ShiftBalanceRule {
    max_daily_shift_difference: usize,
}

impl ShiftBalanceRule {
    pub fn new(max_daily_shift_difference: usize) -> Self {
        Self {
            max_daily_shift_difference,
        }
    }

    /// Count how many staff are assigned to each shift type on a given date
    fn count_shifts_on_date(&self, context: &AssignmentContext) -> HashMap<ShiftType, usize> {
        let mut counts = HashMap::new();
        counts.insert(ShiftType::Morning, 0);
        counts.insert(ShiftType::Evening, 0);
        counts.insert(ShiftType::DayOff, 0);

        for staff_assignments in context.assignments.values() {
            if let Some(shift) = staff_assignments.get(&context.date) {
                *counts.entry(*shift).or_insert(0) += 1;
            }
        }

        counts
    }
}

impl Rule for ShiftBalanceRule {
    fn validate(&self, context: &AssignmentContext) -> DomainResult<()> {
        // Day off doesn't affect shift balance
        if context.shift == ShiftType::DayOff {
            return Ok(());
        }

        let mut counts = self.count_shifts_on_date(context);

        // Simulate adding this assignment
        *counts.entry(context.shift).or_insert(0) += 1;

        let morning_count = *counts.get(&ShiftType::Morning).unwrap_or(&0);
        let evening_count = *counts.get(&ShiftType::Evening).unwrap_or(&0);

        let diff = morning_count.abs_diff(evening_count);

        if diff > self.max_daily_shift_difference {
            return Err(DomainError::InvalidInput(format!(
                "Assigning {} shift on {} would create imbalance: {} morning vs {} evening (max difference: {})",
                match context.shift {
                    ShiftType::Morning => "morning",
                    ShiftType::Evening => "evening",
                    _ => "unknown",
                },
                context.date,
                morning_count,
                evening_count,
                self.max_daily_shift_difference
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_shift_balance_violation() {
        let rule = ShiftBalanceRule::new(1);
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Two staff already assigned to morning shift
        let mut assignments = HashMap::new();

        let staff1 = Uuid::new_v4();
        let mut staff1_assignments = HashMap::new();
        staff1_assignments.insert(date, ShiftType::Morning);
        assignments.insert(staff1, staff1_assignments);

        let staff2 = Uuid::new_v4();
        let mut staff2_assignments = HashMap::new();
        staff2_assignments.insert(date, ShiftType::Morning);
        assignments.insert(staff2, staff2_assignments);

        // Try to assign 3rd morning shift - would be 3 morning vs 0 evening (diff = 3 > 1)
        let staff3 = Uuid::new_v4();
        let context = AssignmentContext {
            assignments,
            staff_id: staff3,
            date,
            shift: ShiftType::Morning,
        };

        assert!(rule.validate(&context).is_err());
    }

    #[test]
    fn test_shift_balance_allowed() {
        let rule = ShiftBalanceRule::new(1);
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Two staff: one morning, one evening
        let mut assignments = HashMap::new();

        let staff1 = Uuid::new_v4();
        let mut staff1_assignments = HashMap::new();
        staff1_assignments.insert(date, ShiftType::Morning);
        assignments.insert(staff1, staff1_assignments);

        let staff2 = Uuid::new_v4();
        let mut staff2_assignments = HashMap::new();
        staff2_assignments.insert(date, ShiftType::Evening);
        assignments.insert(staff2, staff2_assignments);

        // Assigning evening is OK (would be 1 morning vs 2 evening, diff = 1)
        let staff3 = Uuid::new_v4();
        let context = AssignmentContext {
            assignments,
            staff_id: staff3,
            date,
            shift: ShiftType::Evening,
        };

        assert!(rule.validate(&context).is_ok());
    }
}

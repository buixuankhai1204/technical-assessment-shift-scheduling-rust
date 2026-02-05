use super::{AssignmentContext, Rule};
use chrono::{Datelike, NaiveDate};
use shared::{DomainError, DomainResult, ShiftType};

/// Rule: Staff must have minimum days off per week
pub struct MinDaysOffRule {
    min_days_off: usize,
}

impl MinDaysOffRule {
    pub fn new(min_days_off: usize) -> Self {
        Self { min_days_off }
    }

    /// Get the Monday of the week containing the given date
    fn get_week_start(&self, date: NaiveDate) -> NaiveDate {
        let weekday = date.weekday().num_days_from_monday();
        date.checked_sub_signed(chrono::Duration::days(weekday as i64))
            .unwrap_or(date)
    }

    /// Count days off for a staff member in a given week
    fn count_days_off_in_week(&self, context: &AssignmentContext, week_start: NaiveDate) -> usize {
        let staff_assignments = match context.assignments.get(&context.staff_id) {
            Some(assignments) => assignments,
            None => return 0,
        };

        let mut count = 0;
        for day_offset in 0..7 {
            if let Some(date) = week_start.checked_add_signed(chrono::Duration::days(day_offset)) {
                if let Some(shift) = staff_assignments.get(&date) {
                    if *shift == ShiftType::DayOff {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Count remaining unassigned days in the week after the given date
    fn count_remaining_days_in_week(&self, date: NaiveDate, week_start: NaiveDate) -> usize {
        let week_end = week_start
            .checked_add_signed(chrono::Duration::days(6))
            .unwrap_or(date);

        if date > week_end {
            return 0;
        }

        let days_diff = (week_end - date).num_days();
        days_diff.max(0) as usize
    }
}

impl Rule for MinDaysOffRule {
    fn validate(&self, context: &AssignmentContext) -> DomainResult<()> {
        // Only validate when assigning work shifts (not day off)
        if context.shift == ShiftType::DayOff {
            return Ok(());
        }

        let week_start = self.get_week_start(context.date);
        let current_days_off = self.count_days_off_in_week(context, week_start);
        let remaining_days = self.count_remaining_days_in_week(context.date, week_start);
        let max_possible_days_off = current_days_off + remaining_days;

        if max_possible_days_off < self.min_days_off {
            return Err(DomainError::InvalidInput(format!(
                "Assigning work shift on {} would make it impossible to meet minimum {} days off per week",
                context.date, self.min_days_off
            )));
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "MinDaysOff"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_min_days_off_violation() {
        let rule = MinDaysOffRule::new(2);
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let sunday = monday.checked_add_signed(chrono::Duration::days(6)).unwrap();

        // Staff has worked all days except Sunday
        let mut assignments = HashMap::new();
        let mut staff_assignments = HashMap::new();
        for day_offset in 0..6 {
            let date = monday.checked_add_signed(chrono::Duration::days(day_offset)).unwrap();
            staff_assignments.insert(date, ShiftType::Morning);
        }
        assignments.insert(staff_id, staff_assignments);

        // Try to assign work on Sunday - should fail (only 0 days off possible, need 2)
        let context = AssignmentContext {
            assignments,
            staff_id,
            date: sunday,
            shift: ShiftType::Morning,
        };

        assert!(rule.validate(&context).is_err());
    }

    #[test]
    fn test_min_days_off_allowed() {
        let rule = MinDaysOffRule::new(2);
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let assignments = HashMap::new();

        // Assigning work on Monday is OK (6 days remaining, can still get 2 days off)
        let context = AssignmentContext {
            assignments,
            staff_id,
            date: monday,
            shift: ShiftType::Morning,
        };

        assert!(rule.validate(&context).is_ok());
    }
}

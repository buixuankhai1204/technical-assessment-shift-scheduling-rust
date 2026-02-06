use super::{AssignmentContext, Rule};
use chrono::{Datelike, NaiveDate};
use shared::{DomainError, DomainResult, ShiftType};

/// Rule: Staff cannot exceed maximum days off per week
pub struct MaxDaysOffRule {
    max_days_off: usize,
}

impl MaxDaysOffRule {
    pub fn new(max_days_off: usize) -> Self {
        Self { max_days_off }
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
}

impl Rule for MaxDaysOffRule {
    fn validate(&self, context: &AssignmentContext) -> DomainResult<()> {
        // Only validate when assigning day off
        if context.shift != ShiftType::DayOff {
            return Ok(());
        }

        let week_start = self.get_week_start(context.date);
        let current_days_off = self.count_days_off_in_week(context, week_start);

        if current_days_off + 1 > self.max_days_off {
            return Err(DomainError::InvalidInput(format!(
                "Assigning day off on {} would exceed maximum {} days off per week",
                context.date, self.max_days_off
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_max_days_off_violation() {
        let rule = MaxDaysOffRule::new(2);
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let wednesday = monday
            .checked_add_signed(chrono::Duration::days(2))
            .unwrap();

        // Staff already has 2 days off
        let mut assignments = HashMap::new();
        let mut staff_assignments = HashMap::new();
        staff_assignments.insert(monday, ShiftType::DayOff);
        let tuesday = monday.succ_opt().unwrap();
        staff_assignments.insert(tuesday, ShiftType::DayOff);
        assignments.insert(staff_id, staff_assignments);

        // Try to assign 3rd day off - should fail
        let context = AssignmentContext {
            assignments,
            staff_id,
            date: wednesday,
            shift: ShiftType::DayOff,
        };

        assert!(rule.validate(&context).is_err());
    }

    #[test]
    fn test_max_days_off_allowed() {
        let rule = MaxDaysOffRule::new(2);
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tuesday = monday.succ_opt().unwrap();

        // Staff has 1 day off
        let mut assignments = HashMap::new();
        let mut staff_assignments = HashMap::new();
        staff_assignments.insert(monday, ShiftType::DayOff);
        assignments.insert(staff_id, staff_assignments);

        // Assigning 2nd day off is OK
        let context = AssignmentContext {
            assignments,
            staff_id,
            date: tuesday,
            shift: ShiftType::DayOff,
        };

        assert!(rule.validate(&context).is_ok());
    }
}

use super::{AssignmentContext, Rule};
use shared::{DomainError, DomainResult, ShiftType};

/// Rule: Staff cannot work morning shift after evening shift
pub struct NoMorningAfterEveningRule;

impl NoMorningAfterEveningRule {
    pub fn new() -> Self {
        Self
    }

    /// Check if previous day was evening shift
    fn get_previous_shift(&self, context: &AssignmentContext) -> Option<ShiftType> {
        let previous_date = context.date.pred_opt()?;
        context
            .assignments
            .get(&context.staff_id)?
            .get(&previous_date)
            .copied()
    }

    /// Check if next day is morning shift
    fn get_next_shift(&self, context: &AssignmentContext) -> Option<ShiftType> {
        let next_date = context.date.succ_opt()?;
        context
            .assignments
            .get(&context.staff_id)?
            .get(&next_date)
            .copied()
    }
}

impl Rule for NoMorningAfterEveningRule {
    fn validate(&self, context: &AssignmentContext) -> DomainResult<()> {
        // If assigning morning shift, check if previous day was evening
        if context.shift == ShiftType::Morning {
            if let Some(previous_shift) = self.get_previous_shift(context) {
                if previous_shift == ShiftType::Evening {
                    return Err(DomainError::InvalidInput(format!(
                        "Cannot assign morning shift on {} after evening shift on previous day",
                        context.date
                    )));
                }
            }
        }

        // If assigning evening shift, check if next day is morning
        if context.shift == ShiftType::Evening {
            if let Some(next_shift) = self.get_next_shift(context) {
                if next_shift == ShiftType::Morning {
                    return Err(DomainError::InvalidInput(format!(
                        "Cannot assign evening shift on {} before morning shift on next day",
                        context.date
                    )));
                }
            }
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
    fn test_morning_after_evening_violation() {
        let rule = NoMorningAfterEveningRule::new();
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tuesday = monday.succ_opt().unwrap();

        let mut assignments = HashMap::new();
        let mut staff_assignments = HashMap::new();
        staff_assignments.insert(monday, ShiftType::Evening);
        assignments.insert(staff_id, staff_assignments);

        let context = AssignmentContext {
            assignments,
            staff_id,
            date: tuesday,
            shift: ShiftType::Morning,
        };

        assert!(rule.validate(&context).is_err());
    }

    #[test]
    fn test_morning_after_morning_allowed() {
        let rule = NoMorningAfterEveningRule::new();
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tuesday = monday.succ_opt().unwrap();

        let mut assignments = HashMap::new();
        let mut staff_assignments = HashMap::new();
        staff_assignments.insert(monday, ShiftType::Morning);
        assignments.insert(staff_id, staff_assignments);

        let context = AssignmentContext {
            assignments,
            staff_id,
            date: tuesday,
            shift: ShiftType::Morning,
        };

        assert!(rule.validate(&context).is_ok());
    }
}

use chrono::NaiveDate;
use shared::{DomainError, DomainResult, ShiftType};
use uuid::Uuid;

use super::scheduling_rules::{ScheduleState, SchedulingRules};

/// Greedy scheduler implementation
pub struct GreedyScheduler {
    rules: SchedulingRules,
}

impl GreedyScheduler {
    pub fn new(rules: SchedulingRules) -> Self {
        Self { rules }
    }

    /// Generate a 28-day schedule starting from the given Monday
    pub fn generate_schedule(
        &self,
        staff_ids: Vec<Uuid>,
        start_date: NaiveDate,
    ) -> DomainResult<ScheduleState> {
        // Validate that start_date is a Monday
        if start_date.weekday().num_days_from_monday() != 0 {
            return Err(DomainError::ValidationError(
                "Schedule must start on a Monday".to_string(),
            ));
        }

        if staff_ids.is_empty() {
            return Err(DomainError::ValidationError(
                "At least one staff member is required".to_string(),
            ));
        }

        let mut state = ScheduleState::new();
        let period_days = 28;

        // Greedy algorithm: for each day, assign shifts to staff members
        for day_offset in 0..period_days {
            let current_date = start_date
                .checked_add_signed(chrono::Duration::days(day_offset))
                .ok_or_else(|| DomainError::ValidationError("Invalid date".to_string()))?;

            self.assign_shifts_for_day(&mut state, &staff_ids, current_date)?;
        }

        // Validate final schedule meets all weekly requirements
        self.validate_weekly_requirements(&state, &staff_ids, start_date)?;

        Ok(state)
    }

    /// Assign shifts for a single day
    fn assign_shifts_for_day(
        &self,
        state: &mut ScheduleState,
        staff_ids: &[Uuid],
        date: NaiveDate,
    ) -> DomainResult<()> {
        // Try to balance morning and evening shifts first
        let mut unassigned_staff: Vec<Uuid> = staff_ids.to_vec();

        // Assign morning shifts
        let target_morning = unassigned_staff.len() / 3;
        let morning_assigned = self.assign_shift_type(
            state,
            &mut unassigned_staff,
            date,
            ShiftType::Morning,
            target_morning,
        )?;

        // Assign evening shifts
        let target_evening = (unassigned_staff.len() - morning_assigned) / 2;
        let _evening_assigned = self.assign_shift_type(
            state,
            &mut unassigned_staff,
            date,
            ShiftType::Evening,
            target_evening,
        )?;

        // Remaining staff get day off
        for staff_id in unassigned_staff {
            if self.can_assign(state, staff_id, date, ShiftType::DayOff) {
                state.assign(staff_id, date, ShiftType::DayOff);
            } else {
                // If can't assign day off (would violate max days off), assign to a work shift
                if self.can_assign(state, staff_id, date, ShiftType::Morning) {
                    state.assign(staff_id, date, ShiftType::Morning);
                } else if self.can_assign(state, staff_id, date, ShiftType::Evening) {
                    state.assign(staff_id, date, ShiftType::Evening);
                } else {
                    // Fallback: assign anyway but log warning
                    state.assign(staff_id, date, ShiftType::DayOff);
                }
            }
        }

        Ok(())
    }

    /// Try to assign a specific shift type to staff members
    fn assign_shift_type(
        &self,
        state: &mut ScheduleState,
        unassigned_staff: &mut Vec<Uuid>,
        date: NaiveDate,
        shift: ShiftType,
        target_count: usize,
    ) -> DomainResult<usize> {
        let mut assigned_count = 0;
        let mut i = 0;

        while i < unassigned_staff.len() && assigned_count < target_count {
            let staff_id = unassigned_staff[i];

            if self.can_assign(state, staff_id, date, shift) {
                state.assign(staff_id, date, shift);
                unassigned_staff.remove(i);
                assigned_count += 1;
            } else {
                i += 1;
            }
        }

        Ok(assigned_count)
    }

    /// Check if a shift can be assigned without violating rules
    fn can_assign(
        &self,
        state: &ScheduleState,
        staff_id: Uuid,
        date: NaiveDate,
        shift: ShiftType,
    ) -> bool {
        // Check no morning after evening rule
        if state.violates_no_morning_after_evening(staff_id, date, shift) {
            return false;
        }

        // Check days off rules
        if state.violates_days_off_rules(staff_id, date, shift, &self.rules) {
            return false;
        }

        // Check shift balance
        if state.violates_shift_balance(date, shift, &self.rules) {
            return false;
        }

        true
    }

    /// Validate that all staff members meet weekly requirements across the 4-week period
    fn validate_weekly_requirements(
        &self,
        state: &ScheduleState,
        staff_ids: &[Uuid],
        start_date: NaiveDate,
    ) -> DomainResult<()> {
        // Check each of the 4 weeks
        for week in 0..4 {
            let week_start = start_date
                .checked_add_signed(chrono::Duration::days(week * 7))
                .ok_or_else(|| DomainError::ValidationError("Invalid date".to_string()))?;

            for staff_id in staff_ids {
                let days_off = state.count_days_off_in_week(*staff_id, week_start);

                if days_off < self.rules.min_days_off_per_week {
                    return Err(DomainError::ValidationError(format!(
                        "Staff {} has only {} days off in week starting {}, minimum is {}",
                        staff_id, days_off, week_start, self.rules.min_days_off_per_week
                    )));
                }

                if days_off > self.rules.max_days_off_per_week {
                    return Err(DomainError::ValidationError(format!(
                        "Staff {} has {} days off in week starting {}, maximum is {}",
                        staff_id, days_off, week_start, self.rules.max_days_off_per_week
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
    use crate::domain::services::scheduling_rules::SchedulingRules;

    #[test]
    fn test_schedule_generation_starts_on_monday() {
        let rules = SchedulingRules {
            min_days_off_per_week: 1,
            max_days_off_per_week: 2,
            max_daily_shift_difference: 1,
        };
        let scheduler = GreedyScheduler::new(rules);

        let tuesday = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        let staff_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        let result = scheduler.generate_schedule(staff_ids, tuesday);
        assert!(result.is_err());
    }

    #[test]
    fn test_schedule_generation_basic() {
        let rules = SchedulingRules {
            min_days_off_per_week: 1,
            max_days_off_per_week: 2,
            max_daily_shift_difference: 1,
        };
        let scheduler = GreedyScheduler::new(rules);

        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let staff_ids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];

        let result = scheduler.generate_schedule(staff_ids.clone(), monday);
        assert!(result.is_ok());

        let state = result.unwrap();
        let assignments = state.get_all_assignments();

        // Should have 28 days * 4 staff = 112 assignments
        assert_eq!(assignments.len(), 28 * staff_ids.len());
    }
}

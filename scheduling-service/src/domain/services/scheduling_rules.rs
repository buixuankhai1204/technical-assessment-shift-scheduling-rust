use chrono::NaiveDate;
use shared::ShiftType;
use std::collections::HashMap;
use uuid::Uuid;

/// Scheduling rules configuration
#[derive(Debug, Clone)]
pub struct SchedulingRules {
    pub min_days_off_per_week: usize,
    pub max_days_off_per_week: usize,
    pub max_daily_shift_difference: usize,
}

impl SchedulingRules {
    pub fn from_config(config: &crate::infrastructure::config::SchedulingConfig) -> Self {
        Self {
            min_days_off_per_week: config.min_days_off_per_week,
            max_days_off_per_week: config.max_days_off_per_week,
            max_daily_shift_difference: config.max_daily_shift_difference,
        }
    }
}

/// Schedule state tracker for validation
pub struct ScheduleState {
    /// Map of staff_id -> (date -> shift_type)
    assignments: HashMap<Uuid, HashMap<NaiveDate, ShiftType>>,
}

impl ScheduleState {
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
        }
    }

    /// Assign a shift to a staff member
    pub fn assign(&mut self, staff_id: Uuid, date: NaiveDate, shift: ShiftType) {
        self.assignments
            .entry(staff_id)
            .or_insert_with(HashMap::new)
            .insert(date, shift);
    }

    /// Get the shift assigned to a staff member on a specific date
    pub fn get_shift(&self, staff_id: Uuid, date: NaiveDate) -> Option<ShiftType> {
        self.assignments.get(&staff_id)?.get(&date).copied()
    }

    /// Check if assignment violates the "no morning after evening" rule
    pub fn violates_no_morning_after_evening(
        &self,
        staff_id: Uuid,
        date: NaiveDate,
        shift: ShiftType,
    ) -> bool {
        // If assigning morning shift, check if previous day was evening
        if shift == ShiftType::Morning {
            if let Some(previous_date) = date.pred_opt() {
                if let Some(previous_shift) = self.get_shift(staff_id, previous_date) {
                    if previous_shift == ShiftType::Evening {
                        return true;
                    }
                }
            }
        }

        // If assigning evening shift, check if next day is morning
        if shift == ShiftType::Evening {
            if let Some(next_date) = date.succ_opt() {
                if let Some(next_shift) = self.get_shift(staff_id, next_date) {
                    if next_shift == ShiftType::Morning {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Count days off for a staff member in a given week (Monday to Sunday)
    pub fn count_days_off_in_week(&self, staff_id: Uuid, week_start: NaiveDate) -> usize {
        let mut count = 0;
        for day_offset in 0..7 {
            if let Some(date) = week_start.checked_add_signed(chrono::Duration::days(day_offset)) {
                if let Some(shift) = self.get_shift(staff_id, date) {
                    if shift == ShiftType::DayOff {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Check if assigning a day off would violate min/max days off per week rules
    pub fn violates_days_off_rules(
        &self,
        staff_id: Uuid,
        date: NaiveDate,
        shift: ShiftType,
        rules: &SchedulingRules,
    ) -> bool {
        // Find the Monday of the week containing this date
        let week_start = self.get_week_start(date);
        let current_days_off = self.count_days_off_in_week(staff_id, week_start);

        match shift {
            ShiftType::DayOff => {
                // Would exceed maximum?
                current_days_off + 1 > rules.max_days_off_per_week
            }
            _ => {
                // Check if we can still meet minimum requirement
                let remaining_days = self.count_remaining_days_in_week(date, week_start);
                let days_off_still_possible = current_days_off + remaining_days;
                days_off_still_possible < rules.min_days_off_per_week
            }
        }
    }

    /// Get the Monday of the week containing the given date
    fn get_week_start(&self, date: NaiveDate) -> NaiveDate {
        let weekday = date.weekday().num_days_from_monday();
        date.checked_sub_signed(chrono::Duration::days(weekday as i64))
            .unwrap_or(date)
    }

    /// Count how many unassigned days remain in the week after the given date
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

    /// Count how many staff are assigned to each shift type on a given date
    pub fn count_shifts_on_date(&self, date: NaiveDate) -> HashMap<ShiftType, usize> {
        let mut counts = HashMap::new();
        counts.insert(ShiftType::Morning, 0);
        counts.insert(ShiftType::Evening, 0);
        counts.insert(ShiftType::DayOff, 0);

        for staff_assignments in self.assignments.values() {
            if let Some(shift) = staff_assignments.get(&date) {
                *counts.entry(*shift).or_insert(0) += 1;
            }
        }

        counts
    }

    /// Check if assigning this shift would violate the max daily shift difference rule
    pub fn violates_shift_balance(
        &self,
        date: NaiveDate,
        shift: ShiftType,
        rules: &SchedulingRules,
    ) -> bool {
        if shift == ShiftType::DayOff {
            return false; // Day off doesn't affect shift balance
        }

        let mut counts = self.count_shifts_on_date(date);
        *counts.entry(shift).or_insert(0) += 1; // Simulate the assignment

        let morning_count = *counts.get(&ShiftType::Morning).unwrap_or(&0);
        let evening_count = *counts.get(&ShiftType::Evening).unwrap_or(&0);

        let diff = morning_count.abs_diff(evening_count);
        diff > rules.max_daily_shift_difference
    }

    /// Get all assignments as a list
    pub fn get_all_assignments(&self) -> Vec<(Uuid, NaiveDate, ShiftType)> {
        let mut result = Vec::new();
        for (&staff_id, dates) in &self.assignments {
            for (&date, &shift) in dates {
                result.push((staff_id, date, shift));
            }
        }
        result.sort_by_key(|(_, date, _)| *date);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_no_morning_after_evening() {
        let mut state = ScheduleState::new();
        let staff_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Monday

        // Assign evening on Monday
        state.assign(staff_id, date, ShiftType::Evening);

        // Try to assign morning on Tuesday - should violate
        let next_day = date.succ_opt().unwrap();
        assert!(state.violates_no_morning_after_evening(staff_id, next_day, ShiftType::Morning));

        // Assign evening on Tuesday - should not violate
        assert!(!state.violates_no_morning_after_evening(staff_id, next_day, ShiftType::Evening));
    }

    #[test]
    fn test_days_off_counting() {
        let mut state = ScheduleState::new();
        let staff_id = Uuid::new_v4();
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Assign 2 days off in the week
        state.assign(staff_id, monday, ShiftType::DayOff);
        state.assign(
            staff_id,
            monday.checked_add_signed(chrono::Duration::days(2)).unwrap(),
            ShiftType::DayOff,
        );

        assert_eq!(state.count_days_off_in_week(staff_id, monday), 2);
    }

    #[test]
    fn test_shift_balance() {
        let mut state = ScheduleState::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let rules = SchedulingRules {
            min_days_off_per_week: 1,
            max_days_off_per_week: 2,
            max_daily_shift_difference: 1,
        };

        // Assign 2 morning shifts
        state.assign(Uuid::new_v4(), date, ShiftType::Morning);
        state.assign(Uuid::new_v4(), date, ShiftType::Morning);

        // Assigning a 3rd morning would violate (3 morning vs 0 evening = diff of 3 > 1)
        assert!(state.violates_shift_balance(date, ShiftType::Morning, &rules));

        // Assigning evening would not violate (2 morning vs 1 evening = diff of 1 <= 1)
        assert!(!state.violates_shift_balance(date, ShiftType::Evening, &rules));
    }
}

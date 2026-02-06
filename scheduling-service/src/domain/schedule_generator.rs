use chrono::{Datelike, NaiveDate, Utc};
use shared::{DomainError, DomainResult, ShiftType};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::ShiftAssignment;
use crate::domain::rules::{AssignmentContext, Rule};

pub struct ScheduleGenerator {
    rules: Vec<Arc<dyn Rule>>,
}

impl ScheduleGenerator {
    pub fn new(rules: Vec<Arc<dyn Rule>>) -> Self {
        Self { rules }
    }

    /// Generate a 28-day schedule for staff members
    pub fn generate_schedule(
        &self,
        staff_ids: Vec<Uuid>,
        start_date: NaiveDate,
        job_id: Uuid,
    ) -> DomainResult<Vec<ShiftAssignment>> {
        if start_date.weekday().num_days_from_monday() != 0 {
            return Err(DomainError::InvalidInput(
                "Schedule must start on a Monday".to_string(),
            ));
        }

        if staff_ids.is_empty() {
            return Err(DomainError::InvalidInput(
                "At least one staff member is required".to_string(),
            ));
        }

        let mut assignments: HashMap<Uuid, HashMap<NaiveDate, ShiftType>> = HashMap::new();
        let period_days = 28;

        for day_offset in 0..period_days {
            let current_date = start_date
                .checked_add_signed(chrono::Duration::days(day_offset))
                .ok_or_else(|| DomainError::InvalidInput("Invalid date".to_string()))?;

            self.assign_shifts_for_day(&mut assignments, &staff_ids, current_date)?;
        }

        let mut result = Vec::new();
        for (staff_id, staff_assignments) in assignments {
            for (date, shift) in staff_assignments {
                result.push(ShiftAssignment {
                    id: Uuid::new_v4(),
                    schedule_job_id: job_id,
                    staff_id,
                    date,
                    shift,
                    created_at: Utc::now(),
                });
            }
        }

        result.sort_by_key(|a| (a.date, a.staff_id));

        Ok(result)
    }

    /// Validate an assignment against all rules
    fn validate_assignment(&self, context: &AssignmentContext) -> DomainResult<()> {
        for rule in &self.rules {
            rule.validate(context)?;
        }
        Ok(())
    }

    /// Assign shifts for a single day using greedy strategy
    fn assign_shifts_for_day(
        &self,
        assignments: &mut HashMap<Uuid, HashMap<NaiveDate, ShiftType>>,
        staff_ids: &[Uuid],
        date: NaiveDate,
    ) -> DomainResult<()> {
        let mut unassigned_staff: Vec<Uuid> = staff_ids
            .iter()
            .filter(|id| {
                !assignments
                    .get(id)
                    .map(|m| m.contains_key(&date))
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        // Try to balance morning and evening shifts
        let target_morning = unassigned_staff.len() / 3;
        let target_evening = (unassigned_staff.len() - target_morning) / 2;

        self.assign_shift_type(
            assignments,
            &mut unassigned_staff,
            date,
            ShiftType::Morning,
            target_morning,
        )?;

        // Assign evening shifts
        self.assign_shift_type(
            assignments,
            &mut unassigned_staff,
            date,
            ShiftType::Evening,
            target_evening,
        )?;

        // Remaining staff get day off
        for staff_id in unassigned_staff {
            self.try_assign(assignments, staff_id, date, ShiftType::DayOff)?;
        }
        Ok(())
    }

    /// Try to assign a specific shift type to staff members
    fn assign_shift_type(
        &self,
        assignments: &mut HashMap<Uuid, HashMap<NaiveDate, ShiftType>>,
        unassigned_staff: &mut Vec<Uuid>,
        date: NaiveDate,
        shift: ShiftType,
        target_count: usize,
    ) -> DomainResult<()> {
        let mut assigned_count = 0;
        let mut i = 0;

        while i < unassigned_staff.len() && assigned_count < target_count {
            let staff_id = unassigned_staff[i];

            let context = AssignmentContext {
                assignments: assignments.clone(),
                staff_id,
                date,
                shift,
            };

            if self.validate_assignment(&context).is_ok() {
                assignments.entry(staff_id).or_default().insert(date, shift);
                unassigned_staff.remove(i);
                assigned_count += 1;
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// Try to assign a shift to a staff member, with fallback options
    fn try_assign(
        &self,
        assignments: &mut HashMap<Uuid, HashMap<NaiveDate, ShiftType>>,
        staff_id: Uuid,
        date: NaiveDate,
        preferred_shift: ShiftType,
    ) -> DomainResult<()> {
        // Try preferred shift first
        let context = AssignmentContext {
            assignments: assignments.clone(),
            staff_id,
            date,
            shift: preferred_shift,
        };

        if self.validate_assignment(&context).is_ok() {
            assignments
                .entry(staff_id)
                .or_default()
                .insert(date, preferred_shift);
            return Ok(());
        }

        // Try alternative shifts if preferred fails
        let alternatives = match preferred_shift {
            ShiftType::DayOff => vec![ShiftType::Morning, ShiftType::Evening],
            _ => vec![ShiftType::DayOff],
        };

        for alt_shift in alternatives {
            let context = AssignmentContext {
                assignments: assignments.clone(),
                staff_id,
                date,
                shift: alt_shift,
            };

            if self.validate_assignment(&context).is_ok() {
                assignments
                    .entry(staff_id)
                    .or_default()
                    .insert(date, alt_shift);
                return Ok(());
            }
        }

        // If all else fails, assign anyway (best effort)
        assignments
            .entry(staff_id)
            .or_default()
            .insert(date, preferred_shift);

        Ok(())
    }
}

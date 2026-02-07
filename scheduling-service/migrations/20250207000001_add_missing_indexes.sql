-- Add missing indexes for optimal query performance

-- Composite index for schedule_jobs covering common query patterns
-- Helps with queries filtering by staff_group_id and status together
CREATE INDEX IF NOT EXISTS idx_schedule_jobs_group_status ON schedule_jobs(staff_group_id, status);

-- Composite index for schedule_jobs covering period queries with status
CREATE INDEX IF NOT EXISTS idx_schedule_jobs_period_status ON schedule_jobs(period_begin_date, status);

-- Index on schedule_jobs.updated_at for potential time-based queries
CREATE INDEX IF NOT EXISTS idx_schedule_jobs_updated_at ON schedule_jobs(updated_at);

-- Composite index for shift_assignments covering the ORDER BY clause in find_by_job_id
-- ORDER BY date, staff_id
CREATE INDEX IF NOT EXISTS idx_shift_assignments_job_date_staff ON shift_assignments(schedule_job_id, date, staff_id);

-- Index on shift_assignments.shift for potential filtering by shift type
CREATE INDEX IF NOT EXISTS idx_shift_assignments_shift ON shift_assignments(shift);

-- Composite index for potential date range queries with staff
CREATE INDEX IF NOT EXISTS idx_shift_assignments_staff_date ON shift_assignments(staff_id, date);
-- Add missing indexes for optimal query performance

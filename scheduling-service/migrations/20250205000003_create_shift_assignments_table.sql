-- Create shift_assignments table
CREATE TABLE IF NOT EXISTS shift_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_job_id UUID NOT NULL REFERENCES schedule_jobs(id) ON DELETE CASCADE,
    staff_id UUID NOT NULL,
    date DATE NOT NULL,
    shift shift_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_shift_assignments_schedule_job_id ON shift_assignments(schedule_job_id);
CREATE INDEX idx_shift_assignments_staff_id ON shift_assignments(staff_id);
CREATE INDEX idx_shift_assignments_date ON shift_assignments(date);
CREATE UNIQUE INDEX idx_shift_assignments_unique ON shift_assignments(schedule_job_id, staff_id, date);

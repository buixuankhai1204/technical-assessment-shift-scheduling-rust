-- Create schedule_jobs table
CREATE TABLE IF NOT EXISTS schedule_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_group_id UUID NOT NULL,
    period_begin_date DATE NOT NULL,
    status job_status NOT NULL DEFAULT 'PENDING',
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Create indexes
CREATE INDEX idx_schedule_jobs_staff_group_id ON schedule_jobs(staff_group_id);
CREATE INDEX idx_schedule_jobs_status ON schedule_jobs(status);
CREATE INDEX idx_schedule_jobs_period_begin_date ON schedule_jobs(period_begin_date);
CREATE INDEX idx_schedule_jobs_created_at ON schedule_jobs(created_at);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_schedule_jobs_updated_at BEFORE UPDATE ON schedule_jobs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

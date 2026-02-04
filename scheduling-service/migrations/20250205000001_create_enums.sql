-- Create job status enum
CREATE TYPE job_status AS ENUM ('PENDING', 'PROCESSING', 'COMPLETED', 'FAILED');

-- Create shift type enum
CREATE TYPE shift_type AS ENUM ('MORNING', 'EVENING', 'DAY_OFF');

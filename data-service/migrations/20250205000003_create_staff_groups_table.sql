-- Create staff_groups table with hierarchical support
CREATE TABLE IF NOT EXISTS staff_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES staff_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_staff_groups_parent_id ON staff_groups(parent_id);
CREATE INDEX idx_staff_groups_created_at ON staff_groups(created_at);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_staff_groups_updated_at BEFORE UPDATE ON staff_groups
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

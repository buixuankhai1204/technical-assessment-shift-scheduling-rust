-- Create group_memberships table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES staff_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(staff_id, group_id)
);

-- Create indexes
CREATE INDEX idx_group_memberships_staff_id ON group_memberships(staff_id);
CREATE INDEX idx_group_memberships_group_id ON group_memberships(group_id);
CREATE INDEX idx_group_memberships_created_at ON group_memberships(created_at);

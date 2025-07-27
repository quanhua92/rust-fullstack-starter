-- Create task_types table to store supported task types
CREATE TABLE task_types (
    task_type VARCHAR(255) PRIMARY KEY,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add trigger to update updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_task_types_updated_at 
    BEFORE UPDATE ON task_types 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Insert any existing task types from tasks table to avoid constraint violations
INSERT INTO task_types (task_type, description)
SELECT DISTINCT 
    task_type,
    CONCAT('Legacy task type: ', task_type)
FROM tasks
WHERE task_type IS NOT NULL;

-- Note: Foreign key constraint will be added later after workers register their types
-- This allows existing tasks to remain valid while we transition to the new system
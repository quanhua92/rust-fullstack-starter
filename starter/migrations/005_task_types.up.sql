-- Create task_types table to store supported task types
BEGIN;

CREATE TABLE task_types (
    task_type TEXT PRIMARY KEY,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add trigger to update updated_at column (function already exists from migration 001)
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

-- Add foreign key constraint to enforce referential integrity
ALTER TABLE tasks ADD CONSTRAINT fk_tasks_task_type 
    FOREIGN KEY (task_type) REFERENCES task_types(task_type);

COMMIT;
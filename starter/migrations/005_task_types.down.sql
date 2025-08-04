-- Remove foreign key constraint from tasks table
BEGIN;

ALTER TABLE tasks DROP CONSTRAINT IF EXISTS fk_tasks_task_type;

-- Drop trigger first
DROP TRIGGER IF EXISTS update_task_types_updated_at ON task_types;

-- Drop task_types table
DROP TABLE IF EXISTS task_types;

-- Drop function (only if no other tables use it)
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

COMMIT;
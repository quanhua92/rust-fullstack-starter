-- Drop tasks table and related objects
DROP TRIGGER IF EXISTS update_tasks_updated_at ON tasks;
DROP INDEX IF EXISTS idx_tasks_ready_to_run;
DROP INDEX IF EXISTS idx_tasks_created_by;
DROP INDEX IF EXISTS idx_tasks_status_scheduled;
DROP INDEX IF EXISTS idx_tasks_scheduled_at;
DROP INDEX IF EXISTS idx_tasks_created_at;
DROP INDEX IF EXISTS idx_tasks_task_type;
DROP INDEX IF EXISTS idx_tasks_priority;
DROP INDEX IF EXISTS idx_tasks_status;
DROP TABLE IF EXISTS tasks;
DROP TYPE IF EXISTS task_priority;
DROP TYPE IF EXISTS task_status;
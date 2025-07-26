-- Create task status enum
CREATE TYPE task_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled', 'retrying');

-- Create task priority enum  
CREATE TYPE task_priority AS ENUM ('low', 'normal', 'high', 'critical');

-- Tasks table for background job queue
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status task_status NOT NULL DEFAULT 'pending',
    priority task_priority NOT NULL DEFAULT 'normal',
    retry_strategy JSONB NOT NULL DEFAULT '{}',
    max_attempts INTEGER NOT NULL DEFAULT 3,
    current_attempt INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Indexes for tasks
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_priority ON tasks(priority);
CREATE INDEX idx_tasks_task_type ON tasks(task_type);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_tasks_scheduled_at ON tasks(scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX idx_tasks_status_scheduled ON tasks(status, scheduled_at) WHERE status IN ('pending', 'retrying');
CREATE INDEX idx_tasks_created_by ON tasks(created_by) WHERE created_by IS NOT NULL;

-- Create composite index for efficient task fetching
CREATE INDEX idx_tasks_ready_to_run ON tasks(priority DESC, created_at ASC) 
WHERE status IN ('pending', 'retrying');

-- Update trigger for tasks
CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
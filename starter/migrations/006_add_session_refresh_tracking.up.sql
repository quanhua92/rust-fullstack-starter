-- Add last_refreshed_at column to sessions table for token refresh tracking
ALTER TABLE sessions ADD COLUMN last_refreshed_at TIMESTAMPTZ;

-- Create index for last_refreshed_at for efficient queries
CREATE INDEX idx_sessions_last_refreshed_at ON sessions(last_refreshed_at);
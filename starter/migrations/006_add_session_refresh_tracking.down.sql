-- Drop the last_refreshed_at column and its index
DROP INDEX IF EXISTS idx_sessions_last_refreshed_at;
ALTER TABLE sessions DROP COLUMN IF EXISTS last_refreshed_at;
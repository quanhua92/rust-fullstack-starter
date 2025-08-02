-- Drop sessions table and related objects
DROP TRIGGER IF EXISTS update_sessions_updated_at ON sessions;
DROP INDEX IF EXISTS idx_sessions_last_refreshed_at;
DROP INDEX IF EXISTS idx_sessions_is_active;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_token;
DROP INDEX IF EXISTS idx_sessions_user_id;
DROP TABLE IF EXISTS sessions;
-- Drop users table and related objects
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP INDEX IF EXISTS idx_users_role_active;
DROP INDEX IF EXISTS idx_users_created_at;
DROP INDEX IF EXISTS idx_users_role;
DROP INDEX IF EXISTS idx_users_username;
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;
DROP FUNCTION IF EXISTS update_updated_at_column();
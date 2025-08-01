-- Remove role CHECK constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS check_user_role;

-- Remove role-based indexes
DROP INDEX IF EXISTS idx_users_role_active;
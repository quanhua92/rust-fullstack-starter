-- Update any existing invalid or null roles to 'user' (safety measure)
UPDATE users SET role = 'user' WHERE role IS NULL OR role NOT IN ('user', 'moderator', 'admin');

-- Add CHECK constraint for role validation
-- This ensures only valid roles can be stored in the database
ALTER TABLE users ADD CONSTRAINT check_user_role CHECK (role IN ('user', 'moderator', 'admin'));

-- Add index for role-based queries (moderator role will need this)
CREATE INDEX idx_users_role_active ON users(role, is_active) WHERE is_active = true;
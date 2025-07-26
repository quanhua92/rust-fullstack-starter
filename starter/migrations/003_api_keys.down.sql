-- Drop api_keys table and related objects
DROP TRIGGER IF EXISTS update_api_keys_updated_at ON api_keys;
DROP INDEX IF EXISTS idx_api_keys_expires_at;
DROP INDEX IF EXISTS idx_api_keys_is_active;
DROP INDEX IF EXISTS idx_api_keys_key_hash;
DROP INDEX IF EXISTS idx_api_keys_created_by;
DROP TABLE IF EXISTS api_keys;
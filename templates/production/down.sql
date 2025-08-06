-- Drop trigger first
DROP TRIGGER IF EXISTS trigger___MODULE_TABLE___updated_at ON __MODULE_TABLE__;

-- Drop function
DROP FUNCTION IF EXISTS update___MODULE_TABLE___updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx___MODULE_TABLE___search;
DROP INDEX IF EXISTS idx___MODULE_TABLE___metadata;
DROP INDEX IF EXISTS idx___MODULE_TABLE___active_items;
DROP INDEX IF EXISTS idx___MODULE_TABLE___status_priority;
DROP INDEX IF EXISTS idx___MODULE_TABLE___updated_at;
DROP INDEX IF EXISTS idx___MODULE_TABLE___created_at;
DROP INDEX IF EXISTS idx___MODULE_TABLE___priority;
DROP INDEX IF EXISTS idx___MODULE_TABLE___status;
DROP INDEX IF EXISTS idx___MODULE_TABLE___name;

-- Drop table
DROP TABLE IF EXISTS __MODULE_TABLE__;

-- Drop enum type
DROP TYPE IF EXISTS __MODULE_NAME___status;
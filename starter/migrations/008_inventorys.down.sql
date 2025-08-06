-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_inventorys_updated_at ON inventorys;

-- Drop function
DROP FUNCTION IF EXISTS update_inventorys_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_inventorys_search;
DROP INDEX IF EXISTS idx_inventorys_metadata;
DROP INDEX IF EXISTS idx_inventorys_active_items;
DROP INDEX IF EXISTS idx_inventorys_status_priority;
DROP INDEX IF EXISTS idx_inventorys_updated_at;
DROP INDEX IF EXISTS idx_inventorys_created_at;
DROP INDEX IF EXISTS idx_inventorys_priority;
DROP INDEX IF EXISTS idx_inventorys_status;
DROP INDEX IF EXISTS idx_inventorys_name;

-- Drop table
DROP TABLE IF EXISTS inventorys;

-- Drop enum type
DROP TYPE IF EXISTS inventory_status;
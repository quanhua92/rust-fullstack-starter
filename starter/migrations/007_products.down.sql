-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_products_updated_at ON products;

-- Drop function
DROP FUNCTION IF EXISTS update_products_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_products_search;
DROP INDEX IF EXISTS idx_products_metadata;
DROP INDEX IF EXISTS idx_products_active_items;
DROP INDEX IF EXISTS idx_products_status_priority;
DROP INDEX IF EXISTS idx_products_updated_at;
DROP INDEX IF EXISTS idx_products_created_at;
DROP INDEX IF EXISTS idx_products_priority;
DROP INDEX IF EXISTS idx_products_status;
DROP INDEX IF EXISTS idx_products_name;

-- Drop table
DROP TABLE IF EXISTS products;

-- Drop enum type
DROP TYPE IF EXISTS products_status;
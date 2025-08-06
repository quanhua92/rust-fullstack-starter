-- Create inventory status enum
CREATE TYPE inventory_status AS ENUM ('active', 'inactive', 'pending', 'archived');

-- Create inventorys table with advanced features
CREATE TABLE inventorys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    status inventory_status NOT NULL DEFAULT 'active',
    priority INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_inventorys_name ON inventorys(name);
CREATE INDEX idx_inventorys_status ON inventorys(status);
CREATE INDEX idx_inventorys_priority ON inventorys(priority);
CREATE INDEX idx_inventorys_created_at ON inventorys(created_at);
CREATE INDEX idx_inventorys_updated_at ON inventorys(updated_at);

-- Create composite index for common filter combinations
CREATE INDEX idx_inventorys_status_priority ON inventorys(status, priority);

-- Create partial index for active items (most common queries)
CREATE INDEX idx_inventorys_active_items ON inventorys(created_at DESC) 
WHERE status = 'active';

-- Create GIN index for metadata JSON searches
CREATE INDEX idx_inventorys_metadata ON inventorys USING GIN (metadata);

-- Create text search index for full-text search
CREATE INDEX idx_inventorys_search ON inventorys USING GIN (
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, ''))
);

-- Create function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_inventorys_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_inventorys_updated_at
    BEFORE UPDATE ON inventorys
    FOR EACH ROW
    EXECUTE FUNCTION update_inventorys_updated_at();
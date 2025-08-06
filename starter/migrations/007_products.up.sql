-- Create products status enum
CREATE TYPE products_status AS ENUM ('active', 'inactive', 'pending', 'archived');

-- Create products table with advanced features
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    status products_status NOT NULL DEFAULT 'active',
    priority INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_products_name ON products(name);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_priority ON products(priority);
CREATE INDEX idx_products_created_at ON products(created_at);
CREATE INDEX idx_products_updated_at ON products(updated_at);

-- Create composite index for common filter combinations
CREATE INDEX idx_products_status_priority ON products(status, priority);

-- Create partial index for active items (most common queries)
CREATE INDEX idx_products_active_items ON products(created_at DESC) 
WHERE status = 'active';

-- Create GIN index for metadata JSON searches
CREATE INDEX idx_products_metadata ON products USING GIN (metadata);

-- Create text search index for full-text search
CREATE INDEX idx_products_search ON products USING GIN (
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, ''))
);

-- Create function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_products_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger_products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_products_updated_at();
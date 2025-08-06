-- Create __MODULE_NAME__ status enum
CREATE TYPE __MODULE_NAME___status AS ENUM ('active', 'inactive', 'pending', 'archived');

-- Create __MODULE_TABLE__ table with advanced features
CREATE TABLE __MODULE_TABLE__ (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    status __MODULE_NAME___status NOT NULL DEFAULT 'active',
    priority INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx___MODULE_TABLE___name ON __MODULE_TABLE__(name);
CREATE INDEX idx___MODULE_TABLE___status ON __MODULE_TABLE__(status);
CREATE INDEX idx___MODULE_TABLE___priority ON __MODULE_TABLE__(priority);
CREATE INDEX idx___MODULE_TABLE___created_by ON __MODULE_TABLE__(created_by);
CREATE INDEX idx___MODULE_TABLE___created_at ON __MODULE_TABLE__(created_at);
CREATE INDEX idx___MODULE_TABLE___updated_at ON __MODULE_TABLE__(updated_at);

-- Create composite index for common filter combinations
CREATE INDEX idx___MODULE_TABLE___status_priority ON __MODULE_TABLE__(status, priority);

-- Create partial index for active items (most common queries)
CREATE INDEX idx___MODULE_TABLE___active_items ON __MODULE_TABLE__(created_at DESC) 
WHERE status = 'active';

-- Create GIN index for metadata JSON searches
CREATE INDEX idx___MODULE_TABLE___metadata ON __MODULE_TABLE__ USING GIN (metadata);

-- Create text search index for full-text search
CREATE INDEX idx___MODULE_TABLE___search ON __MODULE_TABLE__ USING GIN (
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, ''))
);

-- Create function to automatically update updated_at
CREATE OR REPLACE FUNCTION update___MODULE_TABLE___updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update updated_at
CREATE TRIGGER trigger___MODULE_TABLE___updated_at
    BEFORE UPDATE ON __MODULE_TABLE__
    FOR EACH ROW
    EXECUTE FUNCTION update___MODULE_TABLE___updated_at();
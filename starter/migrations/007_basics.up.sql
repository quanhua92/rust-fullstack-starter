-- Create basics table
CREATE TABLE basics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on name for search performance
CREATE INDEX idx_basics_name ON basics(name);

-- Create index on created_at for sorting
CREATE INDEX idx_basics_created_at ON basics(created_at);
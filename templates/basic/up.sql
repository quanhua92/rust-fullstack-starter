-- Create __MODULE_TABLE__ table
CREATE TABLE __MODULE_TABLE__ (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on name for search performance
CREATE INDEX idx___MODULE_TABLE___name ON __MODULE_TABLE__(name);

-- Create index on created_by for ownership queries  
CREATE INDEX idx___MODULE_TABLE___created_by ON __MODULE_TABLE__(created_by);

-- Create index on created_at for sorting
CREATE INDEX idx___MODULE_TABLE___created_at ON __MODULE_TABLE__(created_at);
-- __MODULE_STRUCT__s table
CREATE TABLE __MODULE_TABLE__ (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for __MODULE_TABLE__
CREATE INDEX idx___MODULE_TABLE___user_id ON __MODULE_TABLE__(user_id);
CREATE INDEX idx___MODULE_TABLE___created_at ON __MODULE_TABLE__(created_at);
CREATE INDEX idx___MODULE_TABLE___title ON __MODULE_TABLE__(title);
CREATE INDEX idx___MODULE_TABLE___active_user ON __MODULE_TABLE__(user_id, is_active) WHERE is_active = true;

-- Update trigger for __MODULE_TABLE__
CREATE TRIGGER update___MODULE_TABLE___updated_at BEFORE UPDATE ON __MODULE_TABLE__
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
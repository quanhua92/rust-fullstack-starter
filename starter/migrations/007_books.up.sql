-- Create books table
CREATE TABLE books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on name for search performance
CREATE INDEX idx_books_name ON books(name);

-- Create index on created_at for sorting
CREATE INDEX idx_books_created_at ON books(created_at);
CREATE TABLE main.categories (
    id BLOB NOT NULL PRIMARY KEY
        CHECK (typeof(id) = 'blob' AND length(id) = 16),
    name TEXT NOT NULL CHECK (name <> ''),
    is_active INTEGER NOT NULL CHECK (is_active IN (0, 1))
) STRICT;

CREATE TABLE main.products (
    id BLOB NOT NULL PRIMARY KEY
        CHECK (typeof(id) = 'blob' AND length(id) = 16),
    name TEXT NOT NULL CHECK(name <> ''),
    description TEXT NOT NULL,
    suggested_price_cents INTEGER NOT NULL CHECK(suggested_price_cents >= 0),
    is_active INTEGER NOT NULL CHECK(is_active IN (0, 1))
) STRICT;

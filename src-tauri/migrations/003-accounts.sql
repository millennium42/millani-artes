CREATE TABLE main.accounts (
    id BLOB NOT NULL PRIMARY KEY
        CHECK (typeof(id) = 'blob' AND length(id) = 16),
    name TEXT NOT NULL CHECK (name <> ''),
    account_type TEXT NOT NULL
        CHECK (account_type IN ('cash', 'bank_pix', 'card', 'other')),
    initial_balance_cents INTEGER NOT NULL
) STRICT;

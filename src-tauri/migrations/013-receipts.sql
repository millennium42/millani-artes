CREATE TABLE main.receipts (
    id BLOB NOT NULL PRIMARY KEY
        CONSTRAINT receipts_id_length CHECK (typeof(id) = 'blob' AND length(id) = 16),
    receivable_id BLOB NOT NULL
        CONSTRAINT receipts_receivable_id_length CHECK (typeof(receivable_id) = 'blob' AND length(receivable_id) = 16),
    account_id BLOB NOT NULL
        CONSTRAINT receipts_account_id_length CHECK (typeof(account_id) = 'blob' AND length(account_id) = 16),
    amount_cents INTEGER NOT NULL
        CONSTRAINT receipts_amount_positive CHECK (amount_cents > 0),
    context TEXT NOT NULL
        CONSTRAINT receipts_context_valid CHECK (context IN ('home', 'factory')),
    CONSTRAINT receipts_receivable_id_fkey
        FOREIGN KEY (receivable_id) REFERENCES receivables (id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    CONSTRAINT receipts_account_id_fkey
        FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE NO ACTION ON UPDATE NO ACTION
) STRICT;

CREATE INDEX main.receipts_receivable_id_idx ON receipts (receivable_id);

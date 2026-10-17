CREATE TABLE main.movements (
    id BLOB NOT NULL PRIMARY KEY CHECK (typeof(id)='blob' AND length(id)=16),
    kind TEXT NOT NULL CHECK (kind IN ('entry','exit','transfer')),
    amount_cents INTEGER NOT NULL,
    context TEXT NOT NULL CHECK (context IN ('home','factory')),
    account_id BLOB CHECK (account_id IS NULL OR (typeof(account_id)='blob' AND length(account_id)=16)),
    source_account_id BLOB CHECK (source_account_id IS NULL OR (typeof(source_account_id)='blob' AND length(source_account_id)=16)),
    destination_account_id BLOB CHECK (destination_account_id IS NULL OR (typeof(destination_account_id)='blob' AND length(destination_account_id)=16)),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    FOREIGN KEY (source_account_id) REFERENCES accounts(id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    FOREIGN KEY (destination_account_id) REFERENCES accounts(id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    CHECK (
        (kind='entry' AND account_id IS NOT NULL AND source_account_id IS NULL AND destination_account_id IS NULL)
        OR (kind='exit' AND account_id IS NOT NULL AND source_account_id IS NULL AND destination_account_id IS NULL)
        OR (kind='transfer' AND account_id IS NULL AND source_account_id IS NOT NULL AND destination_account_id IS NOT NULL)
    )
) STRICT;

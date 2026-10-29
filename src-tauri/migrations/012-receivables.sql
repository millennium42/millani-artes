CREATE TABLE main.receivables (
    id BLOB NOT NULL PRIMARY KEY
        CONSTRAINT receivables_id_length CHECK (typeof(id) = 'blob' AND length(id) = 16),
    sale_id BLOB NOT NULL
        CONSTRAINT receivables_sale_id_length CHECK (typeof(sale_id) = 'blob' AND length(sale_id) = 16),
    amount_cents INTEGER NOT NULL
        CONSTRAINT receivables_amount_positive CHECK (amount_cents > 0),
    customer_id BLOB NOT NULL
        CONSTRAINT receivables_customer_id_length CHECK (typeof(customer_id) = 'blob' AND length(customer_id) = 16),
    due_at_ms INTEGER NOT NULL,
    CONSTRAINT receivables_sale_id_fkey
        FOREIGN KEY (sale_id) REFERENCES sales (id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    CONSTRAINT receivables_customer_id_fkey
        FOREIGN KEY (customer_id) REFERENCES customers (id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    CONSTRAINT receivables_sale_id_unique UNIQUE (sale_id)
) STRICT;

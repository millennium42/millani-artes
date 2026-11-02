CREATE TABLE main.sale_return_items (
    id BLOB NOT NULL PRIMARY KEY
        CONSTRAINT sale_return_items_id_length CHECK (typeof(id) = 'blob' AND length(id) = 16),
    sale_item_id BLOB NOT NULL
        CONSTRAINT sale_return_items_sale_item_id_length
        CHECK (typeof(sale_item_id) = 'blob' AND length(sale_item_id) = 16),
    quantity INTEGER NOT NULL
        CONSTRAINT sale_return_items_quantity_positive CHECK (quantity > 0),
    CONSTRAINT sale_return_items_sale_item_id_fkey
        FOREIGN KEY (sale_item_id) REFERENCES sale_items (id)
        ON DELETE NO ACTION ON UPDATE NO ACTION
) STRICT;

CREATE INDEX main.sale_return_items_sale_item_id_idx
    ON sale_return_items (sale_item_id);

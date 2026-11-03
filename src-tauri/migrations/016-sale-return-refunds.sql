CREATE TABLE main.sale_return_refunds (
    movement_id BLOB NOT NULL PRIMARY KEY
        CONSTRAINT sale_return_refunds_movement_id_length
        CHECK (typeof(movement_id) = 'blob' AND length(movement_id) = 16),
    sale_return_item_id BLOB NOT NULL
        CONSTRAINT sale_return_refunds_return_item_id_length
        CHECK (typeof(sale_return_item_id) = 'blob' AND length(sale_return_item_id) = 16),
    CONSTRAINT sale_return_refunds_movement_fkey
        FOREIGN KEY (movement_id) REFERENCES movements (id)
        ON DELETE NO ACTION ON UPDATE NO ACTION,
    CONSTRAINT sale_return_refunds_return_item_fkey
        FOREIGN KEY (sale_return_item_id) REFERENCES sale_return_items (id)
        ON DELETE NO ACTION ON UPDATE NO ACTION
) STRICT;

CREATE INDEX main.sale_return_refunds_return_item_idx
    ON sale_return_refunds (sale_return_item_id);

CREATE TRIGGER main.sale_return_refunds_require_exit
BEFORE INSERT ON sale_return_refunds
WHEN NOT EXISTS (
    SELECT 1 FROM movements
    WHERE id = NEW.movement_id AND kind = 'exit' AND amount_cents > 0
)
BEGIN
    SELECT RAISE(ABORT, 'refund must reference a positive exit movement');
END;

CREATE TRIGGER main.sale_return_refunds_immutable
BEFORE UPDATE OF kind, amount_cents ON movements
WHEN EXISTS (
    SELECT 1 FROM sale_return_refunds WHERE movement_id = OLD.id
) AND (NEW.kind <> OLD.kind OR NEW.amount_cents <> OLD.amount_cents)
BEGIN
    SELECT RAISE(ABORT, 'refund movement kind and amount are immutable');
END;

CREATE TRIGGER main.sale_return_refunds_no_update
BEFORE UPDATE ON sale_return_refunds
BEGIN
    SELECT RAISE(ABORT, 'refund links are immutable');
END;

CREATE TRIGGER main.sale_return_refunds_no_delete
BEFORE DELETE ON sale_return_refunds
BEGIN
    SELECT RAISE(ABORT, 'refund links cannot be deleted');
END;

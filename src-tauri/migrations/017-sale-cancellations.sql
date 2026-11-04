CREATE TABLE main.sale_cancellations (
    sale_id BLOB NOT NULL PRIMARY KEY
        CONSTRAINT sale_cancellations_sale_id_length
        CHECK (typeof(sale_id) = 'blob' AND length(sale_id) = 16),
    cancelled_at_ms INTEGER NOT NULL
        CONSTRAINT sale_cancellations_timestamp_nonnegative
        CHECK (typeof(cancelled_at_ms) = 'integer' AND cancelled_at_ms >= 0),
    CONSTRAINT sale_cancellations_sale_fkey
        FOREIGN KEY (sale_id) REFERENCES sales (id)
        ON DELETE NO ACTION ON UPDATE NO ACTION
) STRICT;

CREATE TRIGGER main.sale_cancellations_require_finalized_and_settled
BEFORE INSERT ON sale_cancellations
WHEN NOT EXISTS (
    SELECT 1 FROM main.sales WHERE id = NEW.sale_id AND status = 'finalized'
) OR EXISTS (
    SELECT 1
    FROM main.receivables AS receivable
    WHERE receivable.sale_id = NEW.sale_id
      AND receivable.amount_cents >
          COALESCE((
              SELECT SUM(receipt.amount_cents)
              FROM main.receipts AS receipt
              WHERE receipt.receivable_id = receivable.id
          ), 0) + COALESCE((
              SELECT SUM(returned.debt_reduction_cents)
              FROM main.sale_return_items AS returned
              JOIN main.sale_items AS item ON item.id = returned.sale_item_id
              WHERE item.sale_id = NEW.sale_id
          ), 0)
)
BEGIN
    SELECT RAISE(ABORT, 'sale cancellation requires a finalized settled sale');
END;

CREATE TRIGGER main.sale_cancellations_no_update
BEFORE UPDATE ON sale_cancellations
BEGIN
    SELECT RAISE(ABORT, 'sale cancellation is immutable');
END;

CREATE TRIGGER main.sale_cancellations_no_delete
BEFORE DELETE ON sale_cancellations
BEGIN
    SELECT RAISE(ABORT, 'sale cancellation cannot be deleted');
END;

CREATE TRIGGER main.sale_return_items_reject_after_cancel
BEFORE INSERT ON sale_return_items
WHEN EXISTS (
    SELECT 1
    FROM main.sale_items AS item
    JOIN main.sale_cancellations AS cancellation ON cancellation.sale_id = item.sale_id
    WHERE item.id = NEW.sale_item_id
)
BEGIN
    SELECT RAISE(ABORT, 'sale has been cancelled');
END;

CREATE TRIGGER main.sale_return_refunds_reject_after_cancel
BEFORE INSERT ON sale_return_refunds
WHEN EXISTS (
    SELECT 1
    FROM main.sale_return_items AS returned
    JOIN main.sale_items AS item ON item.id = returned.sale_item_id
    JOIN main.sale_cancellations AS cancellation ON cancellation.sale_id = item.sale_id
    WHERE returned.id = NEW.sale_return_item_id
)
BEGIN
    SELECT RAISE(ABORT, 'sale has been cancelled');
END;

CREATE TRIGGER main.receivables_reject_after_cancel
BEFORE INSERT ON receivables
WHEN EXISTS (
    SELECT 1 FROM main.sale_cancellations WHERE sale_id = NEW.sale_id
)
BEGIN
    SELECT RAISE(ABORT, 'sale has been cancelled');
END;

CREATE TRIGGER main.receipts_reject_after_cancel
BEFORE INSERT ON receipts
WHEN EXISTS (
    SELECT 1
    FROM main.receivables AS receivable
    JOIN main.sale_cancellations AS cancellation ON cancellation.sale_id = receivable.sale_id
    WHERE receivable.id = NEW.receivable_id
)
BEGIN
    SELECT RAISE(ABORT, 'sale has been cancelled');
END;

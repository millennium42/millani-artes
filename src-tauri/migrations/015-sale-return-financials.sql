ALTER TABLE main.sale_return_items
    ADD COLUMN return_amount_cents INTEGER NOT NULL DEFAULT 0
        CONSTRAINT sale_return_items_return_amount_nonnegative
        CHECK (return_amount_cents >= 0);

ALTER TABLE main.sale_return_items
    ADD COLUMN debt_reduction_cents INTEGER NOT NULL DEFAULT 0
        CONSTRAINT sale_return_items_debt_reduction_valid
        CHECK (debt_reduction_cents >= 0 AND debt_reduction_cents <= return_amount_cents);

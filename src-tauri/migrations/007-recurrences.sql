CREATE TABLE main.recurrences (
    id BLOB NOT NULL PRIMARY KEY CHECK(typeof(id) = 'blob' AND length(id) = 16),
    name TEXT NOT NULL CHECK(name <> ''),
    kind TEXT NOT NULL CHECK(kind IN ('fixed', 'variable')),
    amount_cents INTEGER,
    context TEXT NOT NULL CHECK(context IN ('home', 'factory')),
    due_day INTEGER NOT NULL CHECK(due_day BETWEEN 1 AND 31),
    CHECK(
        (kind = 'fixed' AND amount_cents IS NOT NULL AND amount_cents > 0) OR
        (kind = 'variable' AND amount_cents IS NULL)
    )
) STRICT;

CREATE TABLE main.recurrence_obligations (
    id BLOB NOT NULL PRIMARY KEY CHECK(typeof(id) = 'blob' AND length(id) = 16),
    recurrence_id BLOB NOT NULL CHECK(typeof(recurrence_id) = 'blob' AND length(recurrence_id) = 16),
    period_year INTEGER NOT NULL CHECK(period_year BETWEEN 1 AND 9999),
    period_month INTEGER NOT NULL CHECK(period_month BETWEEN 1 AND 12),
    due_at_ms INTEGER NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('awaiting_value', 'payable', 'paid', 'overdue')),
    amount_cents INTEGER,
    movement_id BLOB CHECK(movement_id IS NULL OR (typeof(movement_id) = 'blob' AND length(movement_id) = 16)),
    context TEXT NOT NULL CHECK(context IN ('home', 'factory')),
    FOREIGN KEY (recurrence_id) REFERENCES recurrences(id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    FOREIGN KEY (movement_id) REFERENCES movements(id) ON DELETE NO ACTION ON UPDATE NO ACTION,
    UNIQUE(recurrence_id, period_year, period_month),
    UNIQUE(movement_id),
    CHECK(
        (state = 'awaiting_value' AND amount_cents IS NULL AND movement_id IS NULL) OR
        (
            state IN ('payable', 'overdue', 'paid') AND
            amount_cents IS NOT NULL AND
            amount_cents > 0 AND
            (
                (state IN ('payable', 'overdue') AND movement_id IS NULL) OR
                (state = 'paid' AND movement_id IS NOT NULL AND movement_id <> zeroblob(16))
            )
        )
    )
) STRICT;

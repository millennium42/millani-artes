CREATE TABLE main.audit_event (
    event_id BLOB PRIMARY KEY NOT NULL
        CHECK (typeof(event_id) = 'blob' AND length(event_id) = 16 AND event_id <> zeroblob(16)),
    occurred_at_ms INTEGER NOT NULL
        CHECK (typeof(occurred_at_ms) = 'integer' AND occurred_at_ms >= 0),
    operation TEXT NOT NULL
        CHECK (typeof(operation) = 'text' AND length(operation) BETWEEN 1 AND 64 AND substr(operation, 1, 1) GLOB '[a-z]' AND operation NOT GLOB '*[^a-z0-9_]*' AND instr(operation, char(0)) = 0),
    entity TEXT NOT NULL
        CHECK (typeof(entity) = 'text' AND length(entity) BETWEEN 1 AND 64 AND substr(entity, 1, 1) GLOB '[a-z]' AND entity NOT GLOB '*[^a-z0-9_]*' AND instr(entity, char(0)) = 0),
    entity_id BLOB NOT NULL
        CHECK (typeof(entity_id) = 'blob' AND length(entity_id) = 16 AND entity_id <> zeroblob(16))
);

CREATE TRIGGER main.audit_event_no_update
BEFORE UPDATE ON audit_event
BEGIN
    SELECT RAISE(ABORT, 'AUDIT_EVENT_APPEND_ONLY');
END;

CREATE TRIGGER main.audit_event_no_delete
BEFORE DELETE ON audit_event
BEGIN
    SELECT RAISE(ABORT, 'AUDIT_EVENT_APPEND_ONLY');
END;

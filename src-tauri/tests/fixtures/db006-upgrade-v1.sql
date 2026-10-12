-- Synthetic immutable DB-006 version-1 baseline; create a new fixture for future baselines.
CREATE TABLE main.schema_migration (
    version INTEGER PRIMARY KEY CHECK (version > 0) NOT NULL
);
INSERT INTO main.schema_migration(version) VALUES (1);

CREATE TABLE main.upgrade_fixture (
    id INTEGER PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
INSERT INTO main.upgrade_fixture(id, value)
VALUES (1, 'O''Brien'), (2, 'synthetic-before');

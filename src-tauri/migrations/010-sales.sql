CREATE TABLE main.sales (
    id BLOB NOT NULL PRIMARY KEY CHECK(typeof(id) = 'blob' AND length(id) = 16),
    status TEXT NOT NULL CHECK(status IN ('open', 'finalized'))
) STRICT;

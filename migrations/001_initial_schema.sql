-- sqlite schema for a basic blobstore

CREATE TABLE metadata(
    key BLOB PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Separate tables so as to potentially support non-local backends in the
-- future. This would likely work by adding a column to the metadata table that
-- kept track of which storage backend (e.g. local/blobs table, S3, etc) the
-- key was in. That would enable ensuring unique keys across all storage
-- backends as long as those reads/writes were proxied through the service.
-- Alternately, this could have been a single table with `data` being nullable
-- but I'd rather not have that field be nullable.
CREATE TABLE blobs(
    key BLOB PRIMARY KEY NOT NULL,
    data BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

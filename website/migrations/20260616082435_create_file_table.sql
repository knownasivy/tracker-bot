CREATE TABLE file_blobs (
    id UUID PRIMARY KEY,
    file_path TEXT NOT NULL,
    hash BYTEA NOT NULL UNIQUE CHECK (octet_length(hash) = 32),
    size BIGINT NOT NULL
);

CREATE TABLE files (
    id UUID PRIMARY KEY,
    upload_id TEXT NOT NULL UNIQUE,
    blob_id UUID NOT NULL REFERENCES file_blobs(id) ON DELETE RESTRICT,
    original_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);

CREATE INDEX idx_files_blob_id ON files(blob_id);
CREATE INDEX idx_files_upload_id ON files(upload_id);
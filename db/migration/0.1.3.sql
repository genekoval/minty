CREATE TABLE object_preview_error (
    object_id       uuid PRIMARY KEY REFERENCES object ON DELETE CASCADE,
    message         text NOT NULL
);

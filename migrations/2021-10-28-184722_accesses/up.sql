CREATE TABLE accesses (
    document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created timestamptz NOT NULL,
    PRIMARY KEY(document_id,user_id)
);

CREATE INDEX accesses_users_idx ON accesses(user_id,document_id);
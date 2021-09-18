CREATE TABLE documents (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    created timestamptz NOT NULL,
    owner uuid NOT NULL REFERENCES users(id),
    mime text,
    size bigint NOT NULL DEFAULT 0,
    data bytea NOT NULL,
    hash text
);

CREATE INDEX documents_unique_idx ON documents (name, owner, created);

CREATE INDEX documents_owner_idx ON documents (owner);

CREATE INDEX documents_hash_idx ON documents (hash);
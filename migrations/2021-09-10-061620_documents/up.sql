CREATE TABLE documents (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    created timestamptz NOT NULL,
    owner uuid REFERENCES users(id),
    mime text,
    data bytea NOT NULL,
    hash bigint
);

CREATE INDEX documents_unique_idx ON documents (name, owner, created);

CREATE INDEX documents_owner_idx ON documents (owner);

CREATE INDEX documents_hash_idx ON documents (hash);
CREATE TABLE documents (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    created timestamptz NOT NULL,
    owner uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id uuid REFERENCES organizations(id) ON DELETE SET NULL,
    mime text,
    size bigint NOT NULL DEFAULT 0,
    data bytea NOT NULL,
    hash text
);

CREATE INDEX documents_unique_idx ON documents (name, owner, created);

CREATE INDEX documents_owner_idx ON documents (owner);

CREATE INDEX documents_hash_idx ON documents (hash);

-- CREATE INDEX documents_org_idx ON documents (org_id);

create extension  IF NOT EXISTS pg_trgm;

CREATE INDEX document_name_tgrm_idx ON documents USING GIN (name gin_trgm_ops);
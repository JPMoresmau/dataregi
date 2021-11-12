CREATE TABLE organizations (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    created timestamptz NOT NULL
);

CREATE UNIQUE INDEX organizations_name ON organizations (name);

CREATE TABLE members (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created timestamptz NOT NULL,
    org_admin boolean NOT NULL DEFAULT FALSE,
    PRIMARY KEY(user_id,org_id)
);

CREATE INDEX members_org ON members (org_id);


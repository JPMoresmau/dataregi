CREATE TABLE users (
    id uuid PRIMARY KEY,
    email VARCHAR(254) NOT NULL,
    name text NOT NULL,
    created timestamptz NOT NULL,
    last_login timestamptz,
    site_admin boolean not null DEFAULT FALSE
);

CREATE UNIQUE INDEX users_email ON users (email);
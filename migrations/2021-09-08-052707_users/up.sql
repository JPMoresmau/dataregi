CREATE TABLE users (
    id uuid PRIMARY KEY,
    email VARCHAR(254) NOT NULL,
    name text,
    created timestamptz NOT NULL,
    last_login timestamptz
);

CREATE UNIQUE INDEX users_email ON users (email);
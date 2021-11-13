CREATE TABLE users (
    id uuid PRIMARY KEY,
    email VARCHAR(254) NOT NULL,
    name text NOT NULL,
    created timestamptz NOT NULL,
    last_login timestamptz,
    site_admin boolean not null DEFAULT FALSE
);

CREATE UNIQUE INDEX users_email ON users (email);

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

INSERT INTO users values (uuid_generate_v4(),'jpmoresmau@gmail.com','JP Moresmau',now(),null,true);
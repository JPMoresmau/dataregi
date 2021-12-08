create table tokens (
    token VARCHAR(50) not null primary key,
    email VARCHAR(254) NOT NULL,
    created timestamptz NOT NULL
);
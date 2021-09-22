#! /bin/bash
export DATABASE_URL=postgres://postgres:postgres@localhost/dataregi_test
export ROCKET_DATABASES="{postgres_main={ url=\"$DATABASE_URL\"}}"
"/c/Program Files/PostgreSQL/13/bin/psql" -d "$DATABASE_URL" -f tests/test.sql
cargo test $*
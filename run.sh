#! /bin/bash
export DATABASE_URL=postgres://postgres:postgres@localhost/dataregi
export ROCKET_DATABASES="{postgres_main={ url=\"$DATABASE_URL\"}}"
cargo run
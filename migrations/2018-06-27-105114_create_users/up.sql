-- Your SQL goes her-- Your SQL goes here
CREATE EXTENSION
IF NOT EXISTS "uuid-ossp";
CREATE TABLE users
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    email VARCHAR UNIQUE NOT NULL,
    password VARCHAR NOT NULL,
    salt VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true
)

-- Your SQL goes here
CREATE TABLE client_tokens
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    token uuid NOT NULL DEFAULT uuid_generate_v4(),
    store_id uuid NOT NULL,
    referer VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    typ VARCHAR NOT NULL
)

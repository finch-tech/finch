-- Your SQL goes here
CREATE TABLE items
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    store_id uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    price DECIMAL NOT NULL,
    confirmations_required NUMERIC NOT NULL
)

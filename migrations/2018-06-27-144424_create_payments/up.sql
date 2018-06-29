-- Your SQL goes here
CREATE TABLE payments
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    status VARCHAR NOT NULL,
    amount INTEGER NOT NULL,
    store_id uuid NOT NULL,
    created_by uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    index INTEGER NOT NULL,
    eth_address VARCHAR,
    btc_address VARCHAR
)

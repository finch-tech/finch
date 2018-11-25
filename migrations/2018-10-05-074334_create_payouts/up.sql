-- Your SQL goes here
CREATE TABLE payouts
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    status VARCHAR NOT NULL,
    action VARCHAR NOT NULL,
    store_id uuid NOT NULL,
    payment_id uuid NOT NULL,
    typ VARCHAR NOT NULL,
    eth_block_height_required NUMERIC,
    btc_block_height_required NUMERIC,
    transaction_hash VARCHAR,
    created_at TIMESTAMPTZ NOT NULL
)

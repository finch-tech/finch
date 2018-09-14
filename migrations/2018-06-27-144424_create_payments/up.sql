-- Your SQL goes here
CREATE TABLE payments
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    status VARCHAR NOT NULL,
    store_id uuid NOT NULL,
    item_id uuid NOT NULL,
    created_by uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    index INTEGER
    NOT NULL,
    eth_address VARCHAR,
    eth_price NUMERIC,
    btc_address VARCHAR,
    btc_price NUMERIC,
    confirmations_required NUMERIC NOT NULL,
    block_height_required NUMERIC,
    transaction_hash VARCHAR,
    payout_status VARCHAR NOT NULL,
    payout_transaction_hash VARCHAR
)

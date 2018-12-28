-- Your SQL goes here
CREATE TABLE payments
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    status VARCHAR NOT NULL,
    store_id uuid NOT NULL,
    index INTEGER NOT NULL,
    created_by uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    amount_paid DECIMAL,
    transaction_hash VARCHAR,
    fiat VARCHAR NOT NULL,
    price DECIMAL NOT NULL,
    crypto VARCHAR NOT NULL,
    address VARCHAR,
    charge DECIMAL,
    confirmations_required INTEGER NOT NULL,
    block_height_required NUMERIC,
    btc_network VARCHAR,
    eth_network VARCHAR
)

-- Your SQL goes here
CREATE TABLE payments
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    status VARCHAR NOT NULL,
    store_id uuid NOT NULL,
    created_by uuid NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    index INTEGER
    NOT NULL,
    base_price DECIMAL NOT NULL,
    typ VARCHAR NOT NULL,
    address VARCHAR,
    price DECIMAL,
    confirmations_required INTEGER,
    block_height_required NUMERIC,
    transaction_hash VARCHAR
)

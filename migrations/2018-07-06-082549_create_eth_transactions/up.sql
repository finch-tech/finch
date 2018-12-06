-- Your SQL goes here
CREATE TABLE eth_transactions
(
    hash VARCHAR PRIMARY KEY NOT NULL,
    nonce NUMERIC NOT NULL,
    block_hash VARCHAR,
    block_number NUMERIC,
    transaction_index VARCHAR,
    from_address VARCHAR NOT NULL,
    to_address VARCHAR,
    value NUMERIC NOT NULL,
    gas_price NUMERIC NOT NULL,
    gas NUMERIC NOT NULL,
    input VARCHAR NOT NULL
);

-- Your SQL goes here
CREATE TABLE btc_transactions
(
    txid VARCHAR PRIMARY KEY NOT NULL,
    data JSON NOT NULL
    -- hex BYTEA NOT NULL,
    -- hash VARCHAR NOT NULL,
    -- size INTEGER NOT NULL,
    -- vsize INTEGER NOT NULL,
    -- version INTEGER NOT NULL,
    -- locktime INTEGER NOT NULL,
    -- blockhash VARCHAR NOT NULL,
    -- confirmations INTEGER NOT NULL,
    -- time INTEGER NOT NULL,
    -- blocktime INTEGER NOT NULL
);

-- CREATE TABLE btc_transaction_input_script
-- (
--     id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
--     asm VARCHAR NOT NULL,
--     hex BYTEA NOT NULL
-- );

-- CREATE TABLE btc_transaction_input
-- (
--     id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
--     txid VARCHAR NOT NULL REFERENCES btc_transactions(txid),
--     vout INTEGER NOT NULL,
--     script_sig uuid NOT NULL,
--     sequence INTEGER NOT NULL,
--     txinwitness TEXT []
-- );

-- CREATE TABLE btc_transaction_output_script
-- (
--     id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
--     asm VARCHAR NOT NULL,
--     hex BYTEA NOT NULL,
--     req_sigs INTEGER,
--     script_type VARCHAR NOT NULL,
--     address TEXT []
-- );

-- CREATE TABLE btc_transaction_output
-- (
--     id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
--     txid VARCHAR NOT NULL REFERENCES btc_transactions(txid),
--     value NUMERIC NOT NULL,
--     n INTEGER NOT NULL,
--     script uuid NOT NULL
-- );

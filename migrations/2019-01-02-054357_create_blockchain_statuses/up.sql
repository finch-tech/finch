-- Your SQL goes here
CREATE TABLE btc_blockchain_statuses
(
    network VARCHAR PRIMARY KEY NOT NULL,
    block_height NUMERIC NOT NULL
);

CREATE TABLE eth_blockchain_statuses
(
    network VARCHAR PRIMARY KEY NOT NULL,
    block_height NUMERIC NOT NULL
);

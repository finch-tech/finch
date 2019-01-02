-- Your SQL goes here
CREATE TABLE btc_transactions
(
    txid VARCHAR PRIMARY KEY NOT NULL,
    data JSON NOT NULL
);

-- Your SQL goes here
CREATE TABLE app_statuses
(
    id SMALLINT PRIMARY KEY NOT NULL DEFAULT 1,
    eth_block_height NUMERIC
    btc_block_height NUMERIC
);

INSERT INTO app_statuses
DEFAULT VALUES;

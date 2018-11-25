-- Your SQL goes here
CREATE TABLE stores
(
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    owner_id uuid NOT NULL,
    private_key BYTEA NOT NULL,
    public_key BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    eth_payout_addresses TEXT
    [],
    eth_confirmations_required INTEGER,
    btc_payout_addresses TEXT
    [],
    btc_confirmations_required INTEGER,
    mnemonic VARCHAR NOT NULL,
    hd_path VARCHAR NOT NULL,
    base_currency VARCHAR NOT NULL,
    deleted_at TIMESTAMPTZ
)

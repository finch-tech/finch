table! {
    app_statuses (id) {
        id -> Int2,
        block_height -> Nullable<Numeric>,
    }
}

table! {
    client_tokens (id) {
        id -> Uuid,
        name -> Varchar,
        token -> Uuid,
        store_id -> Uuid,
        domain -> Varchar,
        created_at -> Timestamptz,
        typ -> Varchar,
    }
}

table! {
    payments (id) {
        id -> Uuid,
        status -> Varchar,
        store_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        paid_at -> Nullable<Timestamptz>,
        index -> Int4,
        price -> Numeric,
        eth_address -> Nullable<Varchar>,
        eth_price -> Nullable<Numeric>,
        btc_address -> Nullable<Varchar>,
        btc_price -> Nullable<Numeric>,
        eth_confirmations_required -> Numeric,
        eth_block_height_required -> Nullable<Numeric>,
        transaction_hash -> Nullable<Varchar>,
        payout_status -> Varchar,
        payout_transaction_hash -> Nullable<Varchar>,
    }
}

table! {
    stores (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Varchar,
        owner_id -> Uuid,
        private_key -> Bytea,
        public_key -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        eth_payout_addresses -> Array<Text>,
        eth_confirmations_required -> Numeric,
        mnemonic -> Varchar,
        hd_path -> Varchar,
        base_currency -> Varchar,
        currency_api -> Varchar,
        currency_api_key -> Varchar,
        active -> Bool,
    }
}

table! {
    transactions (hash) {
        hash -> Varchar,
        nonce -> Numeric,
        block_hash -> Nullable<Varchar>,
        block_number -> Nullable<Numeric>,
        transaction_index -> Nullable<Varchar>,
        from_address -> Varchar,
        to_address -> Nullable<Varchar>,
        value -> Numeric,
        gas_price -> Numeric,
        gas -> Numeric,
        input -> Varchar,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_verified -> Bool,
        verification_token -> Uuid,
        verification_token_expires_at -> Timestamptz,
        active -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    app_statuses,
    client_tokens,
    payments,
    stores,
    transactions,
    users,
);

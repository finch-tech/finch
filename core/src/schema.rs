table! {
    app_statuses (id) {
        id -> Int2,
        eth_block_height -> Nullable<Numeric>,
        btc_block_height -> Nullable<Numeric>,
    }
}

table! {
    btc_transactions (txid) {
        txid -> Varchar,
        data -> Json,
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
    eth_transactions (hash) {
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
    payments (id) {
        id -> Uuid,
        status -> Varchar,
        store_id -> Uuid,
        index -> Int4,
        created_by -> Uuid,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        paid_at -> Nullable<Timestamptz>,
        amount_paid -> Nullable<Numeric>,
        transaction_hash -> Nullable<Varchar>,
        fiat -> Varchar,
        price -> Numeric,
        crypto -> Varchar,
        address -> Nullable<Varchar>,
        charge -> Nullable<Numeric>,
        confirmations_required -> Int4,
        block_height_required -> Nullable<Numeric>,
        btc_network -> Nullable<Varchar>,
        eth_network -> Nullable<Varchar>,
    }
}

table! {
    payouts (id) {
        id -> Uuid,
        status -> Varchar,
        action -> Varchar,
        store_id -> Uuid,
        payment_id -> Uuid,
        typ -> Varchar,
        block_height_required -> Numeric,
        transaction_hash -> Nullable<Varchar>,
        created_at -> Timestamptz,
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
        eth_payout_addresses -> Nullable<Array<Text>>,
        eth_confirmations_required -> Nullable<Int4>,
        btc_payout_addresses -> Nullable<Array<Text>>,
        btc_confirmations_required -> Nullable<Int4>,
        mnemonic -> Varchar,
        hd_path -> Varchar,
        deleted_at -> Nullable<Timestamptz>,
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
        reset_token -> Nullable<Uuid>,
        reset_token_expires_at -> Nullable<Timestamptz>,
    }
}

allow_tables_to_appear_in_same_query!(
    app_statuses,
    btc_transactions,
    client_tokens,
    eth_transactions,
    payments,
    payouts,
    stores,
    users,
);

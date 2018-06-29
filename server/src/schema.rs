table! {
    client_tokens (id) {
        id -> Uuid,
        name -> Varchar,
        token -> Uuid,
        store_id -> Uuid,
        referer -> Varchar,
        created_at -> Timestamptz,
        typ -> Varchar,
    }
}

table! {
    payments (id) {
        id -> Uuid,
        status -> Varchar,
        amount -> Int4,
        store_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
        paid_at -> Nullable<Timestamptz>,
        index -> Int4,
        eth_address -> Nullable<Varchar>,
        btc_address -> Nullable<Varchar>,
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
        payout_addresses -> Array<Text>,
        mnemonic -> Varchar,
        hd_path -> Varchar,
        active -> Bool,
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
        active -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    client_tokens,
    payments,
    stores,
    users,
);

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
    stores,
    users,
);

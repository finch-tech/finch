use std::convert::From;

use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::{
    postgres::PgExecutorAddr,
    stores::{FindById, FindByIdWithDeleted, FindByOwner, Insert, SoftDelete, Update},
};
use models::{user::User, Error};
use schema::stores;
use types::{currency::Crypto, PrivateKey, PublicKey, H160};

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[table_name = "stores"]
pub struct StorePayload {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub private_key: Option<PrivateKey>,
    pub public_key: Option<PublicKey>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub eth_payout_addresses: Option<Option<Vec<H160>>>,
    pub eth_confirmations_required: Option<Option<i32>>,
    pub btc_payout_addresses: Option<Option<Vec<String>>>,
    pub btc_confirmations_required: Option<Option<i32>>,
    pub mnemonic: Option<String>,
    pub hd_path: Option<String>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}

impl StorePayload {
    pub fn new() -> Self {
        StorePayload {
            id: None,
            name: None,
            description: None,
            owner_id: None,
            private_key: None,
            public_key: None,
            created_at: None,
            updated_at: None,
            eth_payout_addresses: None,
            eth_confirmations_required: None,
            btc_payout_addresses: None,
            btc_confirmations_required: None,
            mnemonic: None,
            hd_path: None,
            deleted_at: None,
        }
    }

    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }

    pub fn set_deleted(&mut self) {
        self.name = Some(String::from(""));
        self.description = Some(String::from(""));
        self.eth_payout_addresses = Some(None);
        self.eth_confirmations_required = Some(None);
        self.deleted_at = Some(Some(Utc::now()));
    }
}

impl From<Store> for StorePayload {
    fn from(store: Store) -> Self {
        StorePayload {
            id: Some(store.id),
            name: Some(store.name),
            description: Some(store.description),
            owner_id: Some(store.owner_id),
            private_key: Some(store.private_key),
            public_key: Some(store.public_key),
            created_at: Some(store.created_at),
            updated_at: Some(store.updated_at),
            eth_payout_addresses: Some(store.eth_payout_addresses),
            eth_confirmations_required: Some(store.eth_confirmations_required),
            btc_payout_addresses: Some(store.btc_payout_addresses),
            btc_confirmations_required: Some(store.btc_confirmations_required),
            mnemonic: Some(store.mnemonic),
            hd_path: Some(store.hd_path),
            deleted_at: Some(store.deleted_at),
        }
    }
}

#[derive(Identifiable, Queryable, Serialize, Associations, Debug)]
#[belongs_to(User, foreign_key = "owner_id")]
pub struct Store {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub eth_payout_addresses: Option<Vec<H160>>,
    pub eth_confirmations_required: Option<i32>,
    pub btc_payout_addresses: Option<Vec<String>>,
    pub btc_confirmations_required: Option<i32>,
    pub mnemonic: String,
    pub hd_path: String,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Store {
    pub fn can_accept(&self, crypto: &Crypto) -> bool {
        match crypto {
            Crypto::Btc => {
                self.btc_payout_addresses.is_some() && self.btc_confirmations_required.is_some()
            }
            Crypto::Eth => {
                self.eth_payout_addresses.is_some() && self.eth_confirmations_required.is_some()
            }
        }
    }

    pub fn insert(
        mut payload: StorePayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Store, Error = Error> {
        payload.set_created_at();
        payload.set_updated_at();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        id: Uuid,
        mut payload: StorePayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Store, Error = Error> {
        payload.set_updated_at();

        (*postgres)
            .send(Update { id, payload })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_owner(
        owner_id: Uuid,
        limit: i64,
        offset: i64,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Vec<Store>, Error = Error> {
        (*postgres)
            .send(FindByOwner {
                owner_id,
                limit,
                offset,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Store, Error = Error> {
        (*postgres)
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_id_with_deleted(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Store, Error = Error> {
        (*postgres)
            .send(FindByIdWithDeleted(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn soft_delete(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = usize, Error = Error> {
        let postgres = postgres.clone();

        postgres
            .send(SoftDelete(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "eth_payout_addresses": self.eth_payout_addresses,
            "eth_confirmations_required": self.eth_confirmations_required,
            "btc_payout_addresses": self.btc_payout_addresses,
            "btc_confirmations_required": self.btc_confirmations_required,
            "public_key": String::from_utf8_lossy(&self.public_key),
            "can_accept_eth": self.can_accept(&Crypto::Eth),
            "can_accept_btc": self.can_accept(&Crypto::Btc),
            "created_at": self.created_at.timestamp(),
            "updated_at": self.updated_at.timestamp(),
        })
    }
}

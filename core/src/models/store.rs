use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use currency_api_client::Api as CurrencyApi;
use db::postgres::PgExecutorAddr;
use db::stores::{Delete, FindById, FindByOwner, Insert, Update};
use models::user::User;
use models::Error;
use schema::stores;
use types::{Currency, H160, PrivateKey, PublicKey, U128};

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[table_name = "stores"]
pub struct StorePayload {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub private_key: Option<PrivateKey>,
    pub public_key: Option<PublicKey>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub eth_payout_addresses: Option<Vec<H160>>,
    pub eth_confirmations_required: Option<U128>,
    pub mnemonic: Option<String>,
    pub hd_path: Option<String>,
    pub base_currency: Option<Currency>,
    pub currency_api: Option<CurrencyApi>,
    pub currency_api_key: Option<String>,
    pub active: bool,
}

impl StorePayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }
}

#[derive(Identifiable, Queryable, Serialize, Associations)]
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
    pub eth_payout_addresses: Vec<H160>,
    pub eth_confirmations_required: U128,
    // TODO: Encryption.
    pub mnemonic: String,
    pub hd_path: String,
    pub base_currency: Currency,
    pub currency_api: CurrencyApi,
    pub currency_api_key: String,
    pub active: bool,
}

impl Store {
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
        store_id: Uuid,
        mut payload: StorePayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Store, Error = Error> {
        payload.set_updated_at();

        (*postgres)
            .send(Update { store_id, payload })
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

    pub fn delete(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = usize, Error = Error> {
        (*postgres)
            .send(Delete(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "created_at": self.created_at.timestamp(),
            "updated_at": self.updated_at.timestamp(),
        })
    }
}

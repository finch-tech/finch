use std::convert::From;

use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use db::payments::{FindAllByEthAddress, FindById, Insert, UpdateById};
use db::postgres::PgExecutorAddr;
use models::item::Item;
use models::store::Store;
use models::transaction::Transaction;
use models::Error;
use schema::payments;
use types::{H160, H256, Status};

#[derive(Debug, Insertable, AsChangeset, Serialize)]
#[table_name = "payments"]
pub struct PaymentPayload {
    pub id: Option<Uuid>,
    pub status: Option<Status>,
    pub store_id: Uuid,
    pub item_id: Uuid,
    pub created_by: Uuid, // AuthClient id
    pub created_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: Option<i32>,
    pub eth_address: Option<H160>,
    pub eth_price: Option<BigDecimal>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
    pub btc_price: Option<BigDecimal>,
    // TODO: Add watch status and expiration
    pub transaction_hash: Option<H256>,
}

impl PaymentPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_paid_at(&mut self) {
        self.paid_at = Some(Utc::now());
    }
}

impl From<Payment> for PaymentPayload {
    fn from(payment: Payment) -> Self {
        PaymentPayload {
            id: Some(payment.id),
            status: Some(payment.status),
            store_id: payment.store_id,
            item_id: payment.item_id,
            created_by: payment.created_by,
            created_at: Some(payment.created_at),
            paid_at: payment.paid_at,
            index: Some(payment.index),
            eth_address: payment.eth_address,
            eth_price: payment.eth_price,
            btc_address: payment.btc_address,
            btc_price: payment.btc_price,
            transaction_hash: payment.transaction_hash,
        }
    }
}

#[derive(Debug, Identifiable, Queryable, Associations, Clone, Serialize, Deserialize)]
#[belongs_to(Store, foreign_key = "store_id")]
#[belongs_to(Item, foreign_key = "item_id")]
pub struct Payment {
    pub id: Uuid,
    pub status: Status,
    pub store_id: Uuid,
    pub item_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: i32,
    pub eth_address: Option<H160>,
    pub eth_price: Option<BigDecimal>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
    pub btc_price: Option<BigDecimal>,
    pub transaction_hash: Option<H256>,
}

impl Payment {
    pub fn insert(
        mut payload: PaymentPayload,
        postgres: PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        payload.set_created_at();

        postgres
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn item(&self, postgres: PgExecutorAddr) -> impl Future<Item = Item, Error = Error> {
        Item::find_by_id(self.item_id, postgres)
    }

    pub fn store(&self, postgres: PgExecutorAddr) -> impl Future<Item = Store, Error = Error> {
        Store::find_by_id(self.store_id, postgres)
    }

    pub fn transaction(
        &self,
        postgres: PgExecutorAddr,
    ) -> impl Future<Item = Transaction, Error = Error> {
        self.to_owned()
            .transaction_hash
            .ok_or(Error::PropertyNotFound)
            .into_future()
            .and_then(|hash| Transaction::find_by_hash(hash, postgres))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        postgres
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update_by_id(
        id: Uuid,
        payload: PaymentPayload,
        postgres: PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        postgres
            .send(UpdateById(id, payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_all_by_eth_address(
        addresses: Vec<H160>,
        postgres: PgExecutorAddr,
    ) -> impl Future<Item = Vec<Payment>, Error = Error> {
        postgres
            .send(FindAllByEthAddress(addresses))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        let mut eth = None;
        let mut btc = None;

        if let Some(ref address) = self.eth_address {
            eth = Some(json!({
                "address": address,
                "price": format!("{}", self.eth_price.clone().unwrap())
            }));
        }

        if let Some(ref address) = self.btc_address {
            btc = Some(json!({
                "address": address,
                "price": format!("{}", self.btc_price.clone().unwrap())
            }));
        }

        json!({
            "id": self.id,
            "status": self.status,
            "store_id": self.store_id,
            "eth": eth,
            "btc": btc
        })
    }
}

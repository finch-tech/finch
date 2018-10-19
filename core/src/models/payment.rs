use std::convert::From;

use bigdecimal::BigDecimal;
use chrono::{prelude::*, Duration};
use futures::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use db::payments::{FindAllByEthAddress, FindById, Insert, UpdateById};
use db::postgres::PgExecutorAddr;
use models::store::Store;
use models::transaction::Transaction;
use models::Error;
use schema::payments;
use types::{PaymentStatus, H160, H256, U128};

#[derive(Debug, Insertable, AsChangeset, Serialize)]
#[table_name = "payments"]
pub struct PaymentPayload {
    pub status: Option<PaymentStatus>,
    pub store_id: Uuid,
    pub created_by: Uuid, // AuthClient id
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: Option<i32>,
    pub price: Option<BigDecimal>,
    pub eth_address: Option<H160>,
    pub eth_price: Option<BigDecimal>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
    pub btc_price: Option<BigDecimal>,
    pub eth_confirmations_required: U128,
    pub eth_block_height_required: Option<U128>,
    pub transaction_hash: Option<H256>,
}

impl PaymentPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_paid_at(&mut self) {
        self.paid_at = Some(Utc::now());
    }

    pub fn set_expires_at(&mut self) {
        self.expires_at = Some(Utc::now() + Duration::seconds(900))
    }
}

impl From<Payment> for PaymentPayload {
    fn from(payment: Payment) -> Self {
        PaymentPayload {
            status: Some(payment.status),
            store_id: payment.store_id,
            created_by: payment.created_by,
            created_at: Some(payment.created_at),
            expires_at: Some(payment.expires_at),
            paid_at: payment.paid_at,
            index: Some(payment.index),
            price: Some(payment.price),
            eth_address: payment.eth_address,
            eth_price: payment.eth_price,
            btc_address: payment.btc_address,
            btc_price: payment.btc_price,
            eth_confirmations_required: payment.eth_confirmations_required,
            eth_block_height_required: payment.eth_block_height_required,
            transaction_hash: payment.transaction_hash,
        }
    }
}

#[derive(Debug, Identifiable, Queryable, Associations, Clone, Serialize, Deserialize)]
#[belongs_to(Store, foreign_key = "store_id")]
pub struct Payment {
    pub id: Uuid,
    pub status: PaymentStatus,
    pub store_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: i32,
    pub price: BigDecimal,
    pub eth_address: Option<H160>,
    pub eth_price: Option<BigDecimal>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
    pub btc_price: Option<BigDecimal>,
    pub eth_confirmations_required: U128,
    pub eth_block_height_required: Option<U128>,
    pub transaction_hash: Option<H256>,
}

impl Payment {
    pub fn insert(
        mut payload: PaymentPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        payload.set_created_at();
        payload.set_expires_at();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn store(&self, postgres: &PgExecutorAddr) -> impl Future<Item = Store, Error = Error> {
        Store::find_by_id(self.store_id, postgres)
    }

    pub fn transaction(
        &self,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Transaction, Error = Error> {
        let postgres = postgres.clone();

        self.transaction_hash
            .ok_or(Error::PropertyNotFound)
            .into_future()
            .and_then(move |hash| Transaction::find_by_hash(hash, &postgres))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        (*postgres)
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update_by_id(
        id: Uuid,
        payload: PaymentPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        (*postgres)
            .send(UpdateById(id, payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_all_by_eth_address(
        addresses: Vec<H160>,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Vec<Payment>, Error = Error> {
        (*postgres)
            .send(FindAllByEthAddress(addresses))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        let mut eth = None;
        let mut btc = None;

        if let Some(ref address) = self.eth_address {
            if let Some(ref price) = self.eth_price {
                eth = Some(json!({
                    "address": address,
                    "price": format!("{}", price)
                }));
            }
        }

        if let Some(ref address) = self.btc_address {
            if let Some(ref price) = self.btc_price {
                btc = Some(json!({
                    "address": address,
                    "price": format!("{}", price)
                }));
            }
        }

        json!({
            "id": self.id,
            "status": self.status,
            "store_id": self.store_id,
            "price": self.price,
            "eth": eth,
            "btc": btc,
            "expires_at": self.expires_at.timestamp()
        })
    }
}

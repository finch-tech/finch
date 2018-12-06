use std::convert::From;

use bigdecimal::BigDecimal;
use chrono::{prelude::*, Duration};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::payments::{FindAllByAddress, FindById, Insert, Update};
use db::postgres::PgExecutorAddr;
use models::store::Store;
use models::Error;
use schema::payments;
use types::{Currency, PaymentStatus, H256, U128};

#[derive(Debug, Insertable, AsChangeset, Serialize)]
#[table_name = "payments"]
pub struct PaymentPayload {
    pub status: Option<PaymentStatus>,
    pub store_id: Uuid,
    pub created_by: Uuid, // AuthClient id
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub paid_at: Option<Option<DateTime<Utc>>>,
    pub index: Option<i32>,
    pub base_price: Option<BigDecimal>,
    pub typ: Option<Currency>,
    pub address: Option<String>,
    pub price: Option<BigDecimal>,
    pub confirmations_required: Option<i32>,
    pub block_height_required: Option<U128>,
    pub transaction_hash: Option<H256>,
}

impl PaymentPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_paid_at(&mut self) {
        self.paid_at = Some(Some(Utc::now()));
    }

    pub fn set_expires_at(&mut self) {
        self.expires_at = Some(Utc::now() + Duration::seconds(3600))
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
            paid_at: Some(payment.paid_at),
            index: Some(payment.index),
            base_price: Some(payment.base_price),
            typ: Some(payment.typ),
            address: payment.address,
            price: payment.price,
            confirmations_required: payment.confirmations_required,
            block_height_required: payment.block_height_required,
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
    pub base_price: BigDecimal,
    pub typ: Currency,
    pub address: Option<String>,
    pub price: Option<BigDecimal>,
    pub confirmations_required: Option<i32>,
    pub block_height_required: Option<U128>,
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
        Store::find_by_id_with_deleted(self.store_id, postgres)
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

    pub fn update(
        id: Uuid,
        payload: PaymentPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        (*postgres)
            .send(Update(id, payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_all_by_address(
        addresses: Vec<String>,
        currency: Currency,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Vec<Payment>, Error = Error> {
        (*postgres)
            .send(FindAllByAddress {
                addresses,
                typ: currency,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        json!({
            "id": self.id,
            "status": self.status,
            "store_id": self.store_id,
            "base_price": self.base_price,
            "type": self.typ,
            "address": self.address,
            "price": self.price,
            "confirmations_required": self.confirmations_required,
            "block_height_required": self.block_height_required,
            "transaction_hash": self.transaction_hash,
            "expires_at": self.expires_at.timestamp()
        })
    }
}

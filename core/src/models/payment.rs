use std::convert::From;

use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::payments::{FindAllByEthAddress, FindById, Insert, UpdateById};
use db::postgres::PgExecutorAddr;
use models::store::Store;
use models::Error;
use schema::payments;
use types::{H160, Status};

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[table_name = "payments"]
pub struct PaymentPayload {
    pub id: Option<Uuid>,
    pub status: Option<Status>,
    pub amount: i32,
    pub store_id: Uuid,
    pub created_by: Uuid, // AuthClient id
    pub created_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: Option<i32>,
    pub eth_address: Option<H160>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
    // TODO: Add watch status and expiration
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
            amount: payment.amount,
            store_id: payment.store_id,
            created_by: payment.created_by,
            created_at: Some(payment.created_at),
            paid_at: payment.paid_at,
            index: Some(payment.index),
            eth_address: payment.eth_address,
            btc_address: payment.btc_address,
        }
    }
}

#[derive(Debug, Identifiable, Queryable, Serialize, Associations, Clone)]
#[belongs_to(Store, foreign_key = "store_id")]
pub struct Payment {
    pub id: Uuid,
    pub status: Status,
    pub amount: i32,
    pub store_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub index: i32,
    pub eth_address: Option<H160>,
    // TODO: Use type for BTC address.
    pub btc_address: Option<String>,
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
        json!({
            "id": self.id,
            "status": self.status,
            "amount": self.amount,
            "store_id": self.store_id,
            "addresses": {
                "eth": self.eth_address,
                "btc": self.btc_address
            }
        })
    }
}

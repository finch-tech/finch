use std::convert::From;

use bigdecimal::BigDecimal;
use chrono::{prelude::*, Duration};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::{
    payments::{FindAllByAddress, FindById, Insert, Update},
    postgres::PgExecutorAddr,
};
use models::{store::Store, Error};
use schema::payments;
use types::{
    bitcoin::Network as BtcNetwork,
    currency::{Crypto, Fiat},
    ethereum::Network as EthNetwork,
    PaymentStatus, H256, U128,
};

#[derive(Debug, Insertable, AsChangeset, Serialize, Clone)]
#[table_name = "payments"]
pub struct PaymentPayload {
    pub status: Option<PaymentStatus>,
    pub store_id: Option<Uuid>,
    pub index: Option<i32>,
    pub created_by: Option<Uuid>, // AuthClient id
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub amount_paid: Option<BigDecimal>,
    pub transaction_hash: Option<H256>,
    pub fiat: Option<Fiat>,
    pub price: Option<BigDecimal>,
    pub crypto: Option<Crypto>,
    pub address: Option<String>,
    pub charge: Option<BigDecimal>,
    pub confirmations_required: Option<i32>,
    pub block_height_required: Option<U128>,
    pub btc_network: Option<BtcNetwork>,
    pub eth_network: Option<EthNetwork>,
    pub identifier: Option<String>,
}

impl PaymentPayload {
    pub fn new() -> Self {
        PaymentPayload {
            status: None,
            store_id: None,
            index: None,
            created_by: None,
            created_at: None,
            expires_at: None,
            paid_at: None,
            amount_paid: None,
            transaction_hash: None,
            fiat: None,
            price: None,
            crypto: None,
            address: None,
            charge: None,
            confirmations_required: None,
            block_height_required: None,
            btc_network: None,
            eth_network: None,
            identifier: None,
        }
    }

    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_paid_at(&mut self) {
        self.paid_at = Some(Utc::now());
    }

    pub fn set_expires_at(&mut self) {
        self.expires_at = Some(Utc::now() + Duration::seconds(3600))
    }
}

impl From<Payment> for PaymentPayload {
    fn from(payment: Payment) -> Self {
        PaymentPayload {
            status: Some(payment.status),
            store_id: Some(payment.store_id),
            index: Some(payment.index),
            created_by: Some(payment.created_by),
            created_at: Some(payment.created_at),
            expires_at: Some(payment.expires_at),
            paid_at: payment.paid_at,
            amount_paid: payment.amount_paid,
            transaction_hash: payment.transaction_hash,
            fiat: Some(payment.fiat),
            price: Some(payment.price),
            crypto: Some(payment.crypto),
            address: Some(payment.address),
            charge: Some(payment.charge),
            confirmations_required: Some(payment.confirmations_required),
            block_height_required: payment.block_height_required,
            btc_network: payment.btc_network,
            eth_network: payment.eth_network,
            identifier: payment.identifier,
        }
    }
}

#[derive(Debug, Identifiable, Queryable, Associations, Clone, Serialize, Deserialize)]
#[belongs_to(Store, foreign_key = "store_id")]
pub struct Payment {
    pub id: Uuid,
    pub status: PaymentStatus,
    #[serde(skip_serializing)]
    pub store_id: Uuid,
    #[serde(skip_serializing)]
    pub index: i32,
    #[serde(skip_serializing)]
    pub created_by: Uuid,
    #[serde(skip_serializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub paid_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub amount_paid: Option<BigDecimal>,
    #[serde(skip_serializing)]
    pub transaction_hash: Option<H256>,
    pub fiat: Fiat,
    pub price: BigDecimal,
    pub crypto: Crypto,
    pub address: String,
    pub charge: BigDecimal,
    pub confirmations_required: i32,
    #[serde(skip_serializing)]
    pub block_height_required: Option<U128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btc_network: Option<BtcNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_network: Option<EthNetwork>,
    pub identifier: Option<String>,
}

impl Payment {
    pub fn insert(
        mut payload: PaymentPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payment, Error = Error> {
        // payload.set_created_at();
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
        crypto: Crypto,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Vec<Payment>, Error = Error> {
        (*postgres)
            .send(FindAllByAddress {
                addresses,
                crypto: crypto,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

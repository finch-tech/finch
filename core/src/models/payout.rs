use std::convert::From;

use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::Future;
use uuid::Uuid;

use db::{
    payouts::{FindAllConfirmed, InsertBtc, InsertEth, Update, UpdateWithPayment},
    postgres::PgExecutorAddr,
};
use models::{
    bitcoin::Transaction as BtcTransaction, ethereum::Transaction as EthTransaction,
    payment::Payment, payment::PaymentPayload, store::Store, Error,
};
use schema::payouts;
use types::{currency::Crypto, PaymentStatus, PayoutAction, PayoutStatus, H256, U128};

#[derive(Debug, Insertable, AsChangeset, Serialize)]
#[table_name = "payouts"]
pub struct PayoutPayload {
    pub status: Option<PayoutStatus>,
    pub action: Option<PayoutAction>,
    pub store_id: Option<Uuid>,
    pub payment_id: Option<Uuid>,
    pub typ: Option<Crypto>,
    pub block_height_required: Option<U128>,
    pub transaction_hash: Option<Option<H256>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl PayoutPayload {
    pub fn new() -> Self {
        PayoutPayload {
            status: None,
            action: None,
            store_id: None,
            payment_id: None,
            typ: None,
            block_height_required: None,
            transaction_hash: None,
            created_at: None,
        }
    }

    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }
}

impl From<Payout> for PayoutPayload {
    fn from(payout: Payout) -> Self {
        PayoutPayload {
            status: Some(payout.status),
            action: Some(payout.action),
            store_id: Some(payout.store_id),
            payment_id: Some(payout.payment_id),
            typ: Some(payout.typ),
            block_height_required: Some(payout.block_height_required),
            transaction_hash: Some(payout.transaction_hash),
            created_at: Some(payout.created_at),
        }
    }
}

#[derive(Debug, Identifiable, Queryable, Associations, Clone, Copy, Serialize, Deserialize)]
#[belongs_to(Store, foreign_key = "store_id")]
#[belongs_to(Payment, foreign_key = "payment_id")]
pub struct Payout {
    pub id: Uuid,
    pub status: PayoutStatus,
    pub action: PayoutAction,
    pub store_id: Uuid,
    pub payment_id: Uuid,
    pub typ: Crypto,
    pub block_height_required: U128,
    pub transaction_hash: Option<H256>,
    pub created_at: DateTime<Utc>,
}

impl Payout {
    pub fn store(&self, postgres: &PgExecutorAddr) -> impl Future<Item = Store, Error = Error> {
        Store::find_by_id_with_deleted(self.store_id, postgres)
    }

    pub fn payment(&self, postgres: &PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
        Payment::find_by_id(self.payment_id, postgres)
    }

    pub fn insert_btc_payout(
        amount_paid: BigDecimal,
        block_height_required: U128,
        payment: Payment,
        transaction: BtcTransaction,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payout, Error = Error> {
        let mut payment_payload = PaymentPayload::from(payment.clone());
        payment_payload.transaction_hash = Some(transaction.hash);
        payment_payload.block_height_required = Some(block_height_required);
        payment_payload.set_paid_at();
        payment_payload.amount_paid = Some(amount_paid.clone());

        let mut payout_payload = PayoutPayload::new();
        payout_payload.status = Some(PayoutStatus::Pending);
        payout_payload.store_id = Some(payment.store_id);
        payout_payload.payment_id = Some(payment.id);
        payout_payload.typ = Some(Crypto::Btc);
        payout_payload.block_height_required = Some(block_height_required);
        payout_payload.set_created_at();

        let charge = payment.charge;
        match payment.status {
            PaymentStatus::Pending | PaymentStatus::Paid => {
                // Paid enough.
                if amount_paid >= charge {
                    payment_payload.status = Some(PaymentStatus::Confirmed);
                    payout_payload.action = Some(PayoutAction::Payout);
                }

                // Insufficient amount paid.
                if amount_paid < charge {
                    payment_payload.status = Some(PaymentStatus::InsufficientAmount);
                    payout_payload.action = Some(PayoutAction::Refund);
                }

            }
            _ => payout_payload.action = Some(PayoutAction::Refund),
        };

        (*postgres)
            .send(InsertBtc {
                payout_payload,
                payment_payload,
                transaction_payload: transaction,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn insert_eth_payout(
        amount_paid: BigDecimal,
        block_height_required: U128,
        payment: Payment,
        transaction: EthTransaction,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payout, Error = Error> {
        let mut payment_payload = PaymentPayload::from(payment.clone());
        payment_payload.transaction_hash = Some(transaction.hash);
        payment_payload.block_height_required = Some(block_height_required);
        payment_payload.set_paid_at();
        payment_payload.amount_paid = Some(amount_paid.clone());

        let mut payout_payload = PayoutPayload::new();
        payout_payload.status = Some(PayoutStatus::Pending);
        payout_payload.store_id = Some(payment.store_id);
        payout_payload.payment_id = Some(payment.id);
        payout_payload.typ = Some(Crypto::Eth);
        payout_payload.block_height_required = Some(block_height_required);
        payout_payload.set_created_at();

        let charge = payment.charge;
        match payment.status {
            PaymentStatus::Pending | PaymentStatus::Paid => {
                // Paid enough.
                if amount_paid >= charge {
                    payment_payload.status = Some(PaymentStatus::Confirmed);
                    payout_payload.action = Some(PayoutAction::Payout);
                }

                // Insufficient amount paid.
                if amount_paid < charge {
                    payment_payload.status = Some(PaymentStatus::InsufficientAmount);
                    payout_payload.action = Some(PayoutAction::Refund);
                }

            }
            _ => payout_payload.action = Some(PayoutAction::Refund),
        };

        (*postgres)
            .send(InsertEth {
                payout_payload,
                payment_payload,
                transaction_payload: transaction,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_all_confirmed(
        block_height: U128,
        typ: Crypto,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Vec<Payout>, Error = Error> {
        (*postgres)
            .send(FindAllConfirmed { block_height, typ })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        id: Uuid,
        payload: PayoutPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payout, Error = Error> {
        (*postgres)
            .send(Update(id, payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update_with_payment(
        id: Uuid,
        payout_payload: PayoutPayload,
        payment_payload: PaymentPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Payout, Error = Error> {
        (*postgres)
            .send(UpdateWithPayment {
                id,
                payout_payload,
                payment_payload,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }
}

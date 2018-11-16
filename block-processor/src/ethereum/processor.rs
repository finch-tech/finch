use std::collections::HashMap;
use std::str::FromStr;

use actix::prelude::*;
use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::{future, stream, Future, Stream};

use core::app_status::{AppStatus, AppStatusPayload};
use core::block::Block;
use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use core::payout::{Payout, PayoutPayload};
use core::transaction::Transaction;
use types::{Currency, PaymentStatus, PayoutAction, PayoutStatus, U128};

use ethereum::errors::Error;

pub type ProcessorAddr = Addr<Processor>;

pub struct Processor {
    pub postgres: PgExecutorAddr,
}

impl Actor for Processor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlock(pub Block);

impl Handler<ProcessBlock> for Processor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, ProcessBlock(block): ProcessBlock, _: &mut Self::Context) -> Self::Result {
        println!("Processing block: {}", block.number.unwrap());
        let postgres = self.postgres.clone();

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();

        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to_address {
                addresses.push(to);
                transactions.insert(to, transaction.clone());
            }
        }

        let block_number = block.number;
        let _postgres = postgres.clone();

        let process = Payment::find_all_by_eth_address(addresses, &postgres)
            .from_err()
            .map(move |payments| stream::iter_ok(payments))
            .flatten_stream()
            .and_then(move |payment| {
                let transaction = transactions.get(&payment.eth_address.unwrap()).unwrap();

                // Prepare payment update.
                let mut payment_payload = PaymentPayload::from(payment.clone());

                // Block height required = transaction's block number + required number of confirmations - 1.
                let block_height_required =
                    block.number.unwrap() + payment.eth_confirmations_required - U128::from(1);

                payment_payload.transaction_hash = Some(Some(transaction.hash));
                payment_payload.eth_block_height_required = Some(Some(block_height_required));
                payment_payload.set_paid_at();

                // Prepare payout object.
                let mut payout_payload = PayoutPayload {
                    action: None,
                    status: Some(PayoutStatus::Pending),
                    store_id: Some(payment.store_id),
                    payment_id: Some(payment.id),
                    typ: Some(Currency::Eth),
                    eth_block_height_required: Some(block_height_required),
                    transaction_hash: None,
                    created_at: None,
                };

                let ether_paid = match BigDecimal::from_str(&format!("{}", transaction.value)) {
                    Ok(value) => value / BigDecimal::from_str("1000000000000000000").unwrap(),
                    Err(_) => {
                        // TODO: Handle error.
                        panic!("Failed to parse transaction amount.");
                    }
                };

                match payment.status {
                    PaymentStatus::Pending => {
                        // Paid enough.
                        if ether_paid >= payment.eth_price.clone().unwrap() {
                            payment_payload.status = Some(PaymentStatus::Paid);
                            payout_payload.action = Some(PayoutAction::Payout);
                        }

                        // Insufficient amount paid.
                        if ether_paid < payment.eth_price.clone().unwrap() {
                            payment_payload.status = Some(PaymentStatus::InsufficientAmount);
                            payout_payload.action = Some(PayoutAction::Refund);
                        }

                        // Expired
                        if payment.expires_at < Utc::now() {
                            payment_payload.status = Some(PaymentStatus::Expired);
                            payout_payload.action = Some(PayoutAction::Refund);
                        }
                    }
                    _ => payout_payload.action = Some(PayoutAction::Refund),
                };

                let transaction = Transaction::insert(transaction.clone(), &postgres).from_err();
                let payment = Payment::update(payment.id, payment_payload, &postgres).from_err();
                let payout = Payout::insert(payout_payload, &postgres).from_err();

                transaction.join3(payment, payout)
            })
            .for_each(move |_| future::ok(()))
            .and_then(move |_| {
                let payload = AppStatusPayload {
                    id: 1,
                    eth_block_height: Some(block_number),
                };

                AppStatus::update(payload, &_postgres).from_err()
            })
            .map(|_| ());

        Box::new(process)
    }
}

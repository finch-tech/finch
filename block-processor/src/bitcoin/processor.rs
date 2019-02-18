use std::{collections::HashMap, str::FromStr};

use actix::prelude::*;
use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::{future, stream, Future, Stream};

use bitcoin::Error;
use core::{
    bitcoin::{Block, BlockchainStatus, BlockchainStatusPayload, Transaction},
    db::postgres::PgExecutorAddr,
    payment::{Payment, PaymentPayload},
    payout::Payout,
};
use types::{bitcoin::Network, currency::Crypto, PaymentStatus, U128};

pub type ProcessorAddr = Addr<Processor>;

pub struct Processor {
    pub network: Network,
    pub postgres: PgExecutorAddr,
}

impl Actor for Processor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessMempoolTransactions(pub Vec<Transaction>);

impl Handler<ProcessMempoolTransactions> for Processor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        ProcessMempoolTransactions(pooled_transactions): ProcessMempoolTransactions,
        _: &mut Self::Context,
    ) -> Self::Result {
        let postgres = self.postgres.clone();

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();
        let mut outputs = HashMap::new();

        for transaction in pooled_transactions {
            for output in transaction.vout.clone() {
                if let Some(output_addresses) = output.script.addresses.clone() {
                    addresses.push(output_addresses[0].clone());
                    transactions.insert(output_addresses[0].clone(), transaction.clone());
                    outputs.insert(output_addresses[0].clone(), output.clone());
                }
            }
        }

        let process = Payment::find_all_by_address(addresses, Crypto::Btc, &postgres)
            .from_err()
            .map(move |payments| stream::iter_ok(payments))
            .flatten_stream()
            .and_then(move |payment| {
                let transaction = transactions.get(&payment.clone().address).unwrap();

                let mut payment_payload = PaymentPayload::from(payment.clone());
                payment_payload.transaction_hash = Some(transaction.hash);
                payment_payload.set_paid_at();

                let vout = outputs.get(&payment.address).unwrap();

                let btc_paid = BigDecimal::from_str(&format!("{}", vout.value))
                    .expect("failed to parse transaction amount");

                // Todo: Verify transaction fee.

                let charge = payment.charge;
                payment_payload.amount_paid = Some(btc_paid.clone());

                match payment.status {
                    PaymentStatus::Pending => {
                        // Paid enough.
                        if btc_paid >= charge {
                            payment_payload.status = Some(PaymentStatus::Paid);
                        }

                        // Insufficient amount paid.
                        if btc_paid < charge {
                            payment_payload.status = Some(PaymentStatus::InsufficientAmount);
                        }

                        // Expired
                        if payment.expires_at < Utc::now() {
                            payment_payload.status = Some(PaymentStatus::Expired);
                        }
                    }
                    _ => (),
                };

                Payment::update(payment.id, payment_payload, &postgres).from_err()
            })
            .for_each(move |_| future::ok(()));

        Box::new(process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlock(pub Block);

impl Handler<ProcessBlock> for Processor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, ProcessBlock(block): ProcessBlock, _: &mut Self::Context) -> Self::Result {
        info!("Processing block: {}", block.height.unwrap());
        let postgres = self.postgres.clone();
        let network = self.network;

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();
        let mut outputs = HashMap::new();

        for transaction in block.clone().transactions.unwrap() {
            for output in transaction.vout.clone() {
                if let Some(output_addresses) = output.script.addresses.clone() {
                    addresses.push(output_addresses[0].clone());
                    transactions.insert(output_addresses[0].clone(), transaction.clone());
                    outputs.insert(output_addresses[0].clone(), output.clone());
                }
            }
        }

        let block_number = block.height;
        let _postgres = postgres.clone();

        let process = Payment::find_all_by_address(addresses, Crypto::Btc, &postgres)
            .from_err()
            .map(move |payments| stream::iter_ok(payments))
            .flatten_stream()
            .and_then(move |payment| {
                let transaction = transactions.get(&payment.clone().address).unwrap();

                let amount_paid = BigDecimal::from_str(&format!(
                    "{}",
                    outputs.get(&payment.address).unwrap().value
                ))
                .expect("failed to parse transaction amount");

                // Block height required = transaction's block number + required number of confirmations - 1.
                let block_height_required = block.height.unwrap()
                    + U128::from(payment.confirmations_required)
                    - U128::from(1);

                Payout::insert_btc_payout(
                    amount_paid,
                    block_height_required,
                    payment,
                    transaction.to_owned(),
                    &postgres,
                )
                .from_err()
            })
            .for_each(move |_| future::ok(()))
            .and_then(move |_| {
                let payload = BlockchainStatusPayload {
                    network: None,
                    block_height: block_number,
                };

                BlockchainStatus::update(network, payload, &_postgres).from_err()
            })
            .map(|_| ());

        Box::new(process)
    }
}

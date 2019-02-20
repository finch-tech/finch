use std::{collections::HashMap, str::FromStr};

use actix::prelude::*;
use bigdecimal::BigDecimal;
use futures::{future, stream, Future, Stream};

use core::{
    db::postgres::PgExecutorAddr,
    ethereum::{Block, BlockchainStatus, BlockchainStatusPayload, Transaction},
    payment::{Payment, PaymentPayload},
    payout::Payout,
};
use ethereum::errors::Error;
use types::{currency::Crypto, ethereum::Network, PaymentStatus, H256, U128};

pub type ProcessorAddr = Addr<Processor>;

pub struct Processor {
    pub network: Network,
    pub postgres: PgExecutorAddr,
}

impl Actor for Processor {
    type Context = Context<Self>;
}

type ProcessedBlockStream = Box<Future<Item = (Vec<String>, HashMap<String, H256>), Error = Error>>;

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlock(pub Block);

impl Handler<ProcessBlock> for Processor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, ProcessBlock(block): ProcessBlock, _: &mut Self::Context) -> Self::Result {
        info!("Processing block: {}", block.number.unwrap());
        let postgres = self.postgres.clone();
        let network = self.network;
        let block_number = block.number;
        let _postgres = postgres.clone();

        let process = stream::iter_ok(block.transactions.clone())
            .filter(|transaction| transaction.to_address.is_some())
            .fold(
                (Vec::new(), HashMap::new()),
                move |(mut addresses, mut tx_hashes), transaction| -> ProcessedBlockStream {
                    let to = transaction.to_address.unwrap();
                    addresses.push(format!("0x{}", to));
                    tx_hashes.insert(format!("0x{}", to), transaction.hash.clone());

                    Box::new(future::ok((addresses, tx_hashes)))
                },
            )
            .and_then(move |(addresses, tx_hashes)| {
                Payment::find_all_by_address(addresses, Crypto::Eth, &postgres)
                    .from_err()
                    .map(move |payments| stream::iter_ok(payments))
                    .flatten_stream()
                    .and_then(move |payment| {
                        let hash = tx_hashes.get(&payment.clone().address).unwrap();
                        let transaction = block
                            .transactions
                            .iter()
                            .find(|tx| tx.hash == *hash)
                            .unwrap();

                        let amount_paid =
                            match BigDecimal::from_str(&format!("{}", transaction.value)) {
                                Ok(value) => {
                                    value / BigDecimal::from_str("1000000000000000000").unwrap()
                                }
                                Err(_) => {
                                    // TODO: Handle error.
                                    panic!("failed to parse transaction amount");
                                }
                            };

                        // Block height required = transaction's block number + required number of confirmations - 1.
                        let block_height_required = block.number.unwrap()
                            + U128::from(payment.confirmations_required)
                            - U128::from(1);

                        Payout::insert_eth_payout(
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
                    .map(|_| ())
            });

        Box::new(process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessPendingTransactions(pub Vec<Transaction>);

impl Handler<ProcessPendingTransactions> for Processor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        ProcessPendingTransactions(pending_transactions): ProcessPendingTransactions,
        _: &mut Self::Context,
    ) -> Self::Result {
        let postgres = self.postgres.clone();

        let process = stream::iter_ok(pending_transactions.clone())
            .filter(|transaction| transaction.to_address.is_some())
            .fold(
                (Vec::new(), HashMap::new()),
                move |(mut addresses, mut tx_hashes), transaction| -> ProcessedBlockStream {
                    let to = transaction.to_address.unwrap();
                    addresses.push(format!("0x{}", to));
                    tx_hashes.insert(format!("0x{}", to), transaction.hash.clone());

                    Box::new(future::ok((addresses, tx_hashes)))
                },
            )
            .and_then(move |(addresses, tx_hashes)| {
                Payment::find_all_by_address(addresses, Crypto::Eth, &postgres)
                    .from_err()
                    .map(move |payments| stream::iter_ok(payments))
                    .flatten_stream()
                    .and_then(move |payment| {
                        let hash = tx_hashes.get(&payment.clone().address).unwrap();
                        let transaction = pending_transactions
                            .iter()
                            .find(|tx| tx.hash == *hash)
                            .unwrap();

                        let mut payment_payload = PaymentPayload::from(payment.clone());

                        let ether_paid =
                            match BigDecimal::from_str(&format!("{}", transaction.value)) {
                                Ok(value) => {
                                    value / BigDecimal::from_str("1000000000000000000").unwrap()
                                }
                                Err(_) => {
                                    // TODO: Handle error.
                                    panic!("failed to parse transaction amount");
                                }
                            };

                        let charge = payment.charge;
                        payment_payload.amount_paid = Some(ether_paid.clone());

                        match payment.status {
                            PaymentStatus::Pending => {
                                // Paid enough.
                                if ether_paid >= charge {
                                    payment_payload.status = Some(PaymentStatus::Paid);
                                }

                                // Insufficient amount paid.
                                if ether_paid < charge {
                                    payment_payload.status =
                                        Some(PaymentStatus::InsufficientAmount);
                                }
                            }
                            _ => (),
                        };

                        Payment::update(payment.id, payment_payload, &postgres).from_err()
                    })
                    .for_each(move |_| future::ok(()))
            });

        Box::new(process)
    }
}

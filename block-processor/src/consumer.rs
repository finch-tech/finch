use std::collections::HashMap;
use std::str::FromStr;

use actix::prelude::*;
use actix_web::actix::spawn;
use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::{future, stream, Future, Stream};

use core::app_status::{AppStatus, AppStatusPayload};
use core::block::Block;
use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use core::payout::{Payout, PayoutPayload};
use core::transaction::Transaction;
use ethereum_client::Client;
use types::{Currency, PaymentStatus, PayoutAction, PayoutStatus, U128};

use errors::Error;

pub type ConsumerAddr = Addr<Consumer>;

pub struct Consumer {
    pub postgres: PgExecutorAddr,
    pub ethereum_rpc_url: String,
    pub skip_missed_blocks: bool,
}

impl Actor for Consumer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlocks {
    pub from: U128,
    pub to: U128,
}

impl Handler<ProcessBlocks> for Consumer {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        ProcessBlocks { from, to }: ProcessBlocks,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let eth_client = Client::new(self.ethereum_rpc_url.clone());

        Box::new(
            stream::unfold(from, move |block_number| {
                if block_number <= to {
                    let next_block_number = block_number.clone() + U128::from(1);
                    Some(future::ok::<_, _>((block_number, next_block_number)))
                } else {
                    None
                }
            }).for_each(move |block_number| {
                let address = address.clone();

                eth_client
                    .get_block_by_number(U128::from(block_number))
                    .from_err()
                    .and_then(move |block| {
                        address
                            .send(ProcessBlock(block))
                            .from_err()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                    })
            }),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct Startup;

impl Handler<Startup> for Consumer {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(&mut self, _: Startup, ctx: &mut Self::Context) -> Self::Result {
        let address = ctx.address();
        let skip_missed_blocks = self.skip_missed_blocks;

        let app_status = AppStatus::find(&self.postgres).from_err();
        let current_block_number = Client::new(self.ethereum_rpc_url.clone())
            .get_block_number()
            .from_err();

        let process = app_status.join(current_block_number).and_then(
            move |(status, current_block_number)| {
                if skip_missed_blocks {
                    println!("Not processing missed blocks.");
                    return future::Either::A(future::ok(current_block_number));
                }

                if let Some(block_height) = status.block_height {
                    if block_height == current_block_number {
                        return future::Either::A(future::ok(current_block_number));
                    }

                    let from = block_height + U128::from(1);

                    println!(
                        "Fetching missed blocks: {} ~ {}",
                        from, current_block_number
                    );

                    future::Either::B(
                        address
                            .send(ProcessBlocks {
                                from,
                                to: current_block_number,
                            })
                            .from_err()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                            .and_then(move |_| {
                                address
                                    .send(Startup)
                                    .from_err()
                                    .and_then(|res| res.map_err(|e| Error::from(e)))
                            }),
                    )
                } else {
                    return future::Either::A(future::ok(current_block_number));
                }
            },
        );

        Box::new(process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlock(pub Block);

impl Handler<ProcessBlock> for Consumer {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, ProcessBlock(block): ProcessBlock, _: &mut Self::Context) -> Self::Result {
        println!("Processing block: {}", block.number.clone().unwrap());
        let postgres = self.postgres.clone();

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();

        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to_address.clone() {
                addresses.push(to.clone());
                transactions.insert(to, transaction.clone());
            }
        }

        let block_number = block.number.clone();
        let _postgres = postgres.clone();

        let process = Payment::find_all_by_eth_address(addresses, &postgres)
            .from_err()
            .map(move |payments| stream::iter_ok(payments))
            .flatten_stream()
            .and_then(move |payment| {


                let transaction = transactions
                    .get(&payment.eth_address.clone().unwrap())
                    .unwrap();

                // Prepare payment update.
                let mut payment_payload = PaymentPayload::from(payment.clone());

                // Block height required = transaction's block number + required number of confirmations - 1.
                let block_height_required = block.number.clone().unwrap()
                    + payment.eth_confirmations_required.clone()
                    - U128::from(1);

                payment_payload.transaction_hash = Some(transaction.hash.clone());
                payment_payload.eth_block_height_required = Some(block_height_required);
                payment_payload.set_paid_at();

                // Prepare payout object.
                let mut payout_payload = PayoutPayload {
                    action: None,
                    status: Some(PayoutStatus::Pending),
                    store_id: Some(payment.store_id),
                    payment_id: Some(payment.id),
                    typ: Some(Currency::Eth),
                    eth_block_height_required: payment_payload.eth_block_height_required.clone(),
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
                        if ether_paid >= payment.clone().eth_price.unwrap() {
                            payment_payload.status = Some(PaymentStatus::Paid);
                            payout_payload.action = Some(PayoutAction::Payout);
                        }

                        // Insufficient amount paid.
                        if ether_paid < payment.clone().eth_price.unwrap() {
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
                let payment = Payment::update_by_id(payment.id, payment_payload, &postgres).from_err();
                let payout = Payout::insert(payout_payload, &postgres).from_err();

                transaction.join3(payment, payout)
            })
            .for_each(move |_| future::ok(()))
            .and_then(move |_| {
                let payload = AppStatusPayload {
                    id: 1,
                    block_height: block_number,
                };

                AppStatus::update(payload, &_postgres).from_err()
            })
            // .from_err()
            .map(|_| ());

        Box::new(process)
    }
}

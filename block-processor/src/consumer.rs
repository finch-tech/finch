use std::collections::HashMap;
use std::str::FromStr;

use actix::prelude::*;
use actix_web::actix::spawn;
use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::Future;

use core::app_status::{AppStatus, AppStatusPayload};
use core::block::Block;
use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use core::transaction::Transaction;
use ethereum_client::Client;
use types::{PaymentStatus, U128};

pub type ConsumerAddr = Addr<Consumer>;

pub struct Consumer {
    pub postgres: PgExecutorAddr,
    pub ethereum_rpc_url: String,
    pub skip_missed_blocks: bool,
}

impl Actor for Consumer {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Context<Self>) {
        println!("Started consumer");
    }
}

impl Consumer {
    pub fn process_block(&self, block: Block) -> impl Future<Item = (), Error = ()> {
        println!("Processing block: {}", block.number.clone().unwrap());

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();

        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to_address.clone() {
                addresses.push(to.clone());
                transactions.insert(to, transaction.clone());
            }
        }

        let postgres = self.postgres.clone();

        Payment::find_all_by_eth_address(addresses, &postgres)
            .map_err(|_| ())
            .and_then(move |payments| {
                for (_, payment) in payments.iter().enumerate() {
                    if let Some(transaction) =
                        transactions.get(&payment.eth_address.clone().unwrap())
                    {
                        let ether_paid =
                            match BigDecimal::from_str(&format!("{}", transaction.value)) {
                                Ok(value) => {
                                    value / BigDecimal::from_str("1000000000000000000").unwrap()
                                }
                                Err(_) => {
                                    // TODO: Handle error.
                                    panic!("Failed to parse transaction amount.");
                                }
                            };

                        let transaction =
                            match Transaction::insert((*transaction).clone(), &postgres).wait() {
                                Ok(transaction) => transaction,
                                Err(_) => {
                                    // TODO: Handle error.
                                    panic!("Failed to insert new transaction to db.");
                                }
                            };

                        let mut payload = PaymentPayload::from(payment.clone());

                        // Block height required = transaction's block number + required number of confirmations - 1.
                        let block_height_required = block.number.clone().unwrap().0
                            + payment.eth_confirmations_required.0
                            - U128::from(1).0;

                        payload.transaction_hash = Some(transaction.hash.clone());
                        payload.eth_block_height_required = Some(U128(block_height_required));
                        payload.set_paid_at();

                        // Paid enough.
                        if ether_paid >= payment.clone().eth_price.unwrap() {
                            payload.status = Some(PaymentStatus::Paid);
                        }

                        // Insufficient amount paid.
                        if ether_paid < payment.clone().eth_price.unwrap() {
                            payload.status = Some(PaymentStatus::InsufficientAmount);
                        }

                        // Expired
                        if payment.expires_at < Utc::now() {
                            payload.status = Some(PaymentStatus::Expired);
                        }

                        match Payment::update_by_id(payment.id, payload, &postgres).wait() {
                            Ok(_) => (),
                            Err(_) => {
                                // TODO: Handle error.
                                panic!("Failed to update payment status.");
                            }
                        };
                    }
                }

                let payload = AppStatusPayload {
                    id: 1,
                    block_height: block.number,
                };

                AppStatus::update(payload, &postgres).map_err(|_| {
                    // TODO: Handle error.
                    panic!("Failed to update app status.");
                })
            })
            .map(|_| ())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Startup(pub Recipient<Ready>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Ready;

impl Handler<Startup> for Consumer {
    type Result = ();

    fn handle(&mut self, Startup(caller): Startup, ctx: &mut Self::Context) -> Self::Result {
        if self.skip_missed_blocks {
            println!("Not processing missed blocks.");
            caller.try_send(Ready).unwrap();
            return;
        }

        let eth_client = Client::new(self.ethereum_rpc_url.clone());

        match AppStatus::find(&self.postgres).wait() {
            Ok(status) => match eth_client.get_block_number().wait() {
                Ok(current_block_number) => {
                    if status.block_height.is_none() {
                        caller.try_send(Ready).unwrap();
                        return;
                    }

                    if let Some(block_height) = status.block_height {
                        if block_height == current_block_number {
                            caller.try_send(Ready).unwrap();
                            return;
                        }

                        println!(
                            "Fetching missed blocks: {} ~ {}",
                            block_height, current_block_number
                        );

                        for x in block_height.0.low_u64() + 1..=current_block_number.0.low_u64() {
                            let block = eth_client
                                .get_block_by_number(U128::from(x))
                                .wait()
                                .expect("Failed to get block by number");

                            match self.process_block(block).wait() {
                                Ok(_) => (),
                                Err(_) => {
                                    // TODO: Handle error.
                                    ()
                                }
                            };
                        }

                        ctx.notify(Startup(caller));
                        return;
                    };
                }
                Err(_) => {
                    panic!();
                }
            },
            Err(_) => panic!(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewBlock(pub Block);

impl Handler<NewBlock> for Consumer {
    type Result = ();

    fn handle(&mut self, NewBlock(block): NewBlock, _: &mut Self::Context) -> Self::Result {
        spawn(self.process_block(block).map_err(|_| ()));
    }
}

use std::collections::HashMap;
use std::str::FromStr;

use actix::prelude::*;
use bigdecimal::BigDecimal;
use futures::Future;

use core::app_status::{AppStatus, AppStatusPayload};
use core::block::Block;
use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use core::transaction::Transaction;
use types::{PaymentStatus, U128};

pub type ConsumerAddr = Addr<Consumer>;

pub struct Consumer {
    pub postgres: PgExecutorAddr,
}

impl Actor for Consumer {
    type Context = SyncContext<Self>;

    fn started(&mut self, _: &mut SyncContext<Self>) {
        println!("Started consumer");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewBlock(pub Block);

impl Handler<NewBlock> for Consumer {
    type Result = ();

    fn handle(&mut self, NewBlock(block): NewBlock, _: &mut Self::Context) -> Self::Result {
        println!("Processing block: {}", block.number.clone().unwrap());

        let mut addresses = Vec::new();
        let mut transactions = HashMap::new();

        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to_address.clone() {
                addresses.push(to.clone());
                transactions.insert(to, transaction.clone());
            }
        }

        // TODO: Get payments that are Pending. find pending by eth address
        if let Ok(payments) = Payment::find_all_by_eth_address(addresses, &self.postgres).wait() {
            for (_, payment) in payments.iter().enumerate() {
                // Start processing payment.id

                if let Some(transaction) = transactions.get(&payment.eth_address.clone().unwrap()) {
                    let ether_paid = match BigDecimal::from_str(&format!("{}", transaction.value)) {
                        Ok(value) => value / BigDecimal::from_str("1000000000000000000").unwrap(),
                        Err(_) => {
                            // TODO: Handle error.
                            continue;
                        }
                    };

                    let transaction =
                        match Transaction::insert((*transaction).clone(), &self.postgres).wait() {
                            Ok(transaction) => transaction,
                            Err(_) => {
                                // TODO: Handle error
                                continue;
                            }
                        };

                    if ether_paid >= payment.clone().eth_price.unwrap() {
                        let mut payload = PaymentPayload::from(payment.clone());

                        let block_height_required =
                            block.number.clone().unwrap().0 + payment.confirmations_required.0;

                        payload.transaction_hash = Some(transaction.hash.clone());
                        payload.status = Some(PaymentStatus::Paid);
                        payload.block_height_required = Some(U128(block_height_required));
                        payload.set_paid_at();

                        match Payment::update_by_id(payment.id, payload, &self.postgres).wait() {
                            Ok(_) => (),
                            Err(_) => {
                                // TODO: Handle error
                                continue;
                            }
                        };
                    }
                }
            }
        };

        let payload = AppStatusPayload {
            id: 1,
            block_height: block.number,
        };

        match AppStatus::update(payload, &self.postgres).wait() {
            Ok(_) => (),
            Err(_) => {
                panic!("Failed to update app status: block_height");
            }
        };
    }
}

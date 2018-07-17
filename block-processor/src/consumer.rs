use std::str::FromStr;

use actix::prelude::*;
use bigdecimal::BigDecimal;
use futures::Future;

use core::block::Block;
use core::db::postgres::PgExecutorAddr;
use core::db::redis::{Publish, RedisExecutorAddr};
use core::payment::{Payment, PaymentPayload};
use core::transaction::Transaction;
use types::Status as PaymentStatus;

pub type ConsumerAddr = Addr<Consumer>;

pub struct Consumer {
    pub redis: RedisExecutorAddr,
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
        let mut addresses = Vec::new();
        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to_address.clone() {
                addresses.push(to);
            }
        }

        if let Ok(payments) =
            Payment::find_all_by_eth_address(addresses, self.postgres.clone()).wait()
        {
            for (_, payment) in payments.iter().enumerate() {
                for (_, transaction) in block.transactions.iter().enumerate() {
                    if let Some(to) = transaction.to_address.clone() {
                        if payment.eth_address.clone().unwrap() == to {
                            let ether =
                                match BigDecimal::from_str(&format!("{}", transaction.value)) {
                                    Ok(value) => {
                                        value / BigDecimal::from_str("1000000000000000000").unwrap()
                                    }
                                    Err(_) => {
                                        // TODO: Handle error.
                                        continue;
                                    }
                                };

                            if ether > payment.clone().eth_price.unwrap() {
                                let mut payload = PaymentPayload::from(payment.clone());
                                payload.status = Some(PaymentStatus::Paid);

                                match Transaction::insert(
                                    (*transaction).clone(),
                                    self.postgres.clone(),
                                ).wait()
                                {
                                    Ok(transaction) => {
                                        payload.transaction_hash = Some(transaction.hash.clone())
                                    }
                                    Err(_) => {
                                        // TODO: Handle error
                                        continue;
                                    }
                                };

                                let payment = match Payment::update_by_id(
                                    payment.id,
                                    payload,
                                    self.postgres.clone(),
                                ).wait()
                                {
                                    Ok(payment) => payment,
                                    Err(_) => {
                                        // TODO: Handle error
                                        continue;
                                    }
                                };

                                self.redis.do_send(Publish {
                                    key: String::from("payment"),
                                    value: json!(payment).to_string(),
                                });
                            }
                        }
                    }
                }
            }
        };
    }
}

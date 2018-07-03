use actix::prelude::*;
use futures::Future;

use core::db::postgres::PgExecutorAddr;
use core::db::redis::{Publish, RedisExecutorAddr};
use core::payment::Payment;
use types::Block;

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
        println!("Payment check on: {:?}", block.number.unwrap().low_u32());

        let mut addresses = Vec::new();
        for (_, transaction) in block.transactions.iter().enumerate() {
            if let Some(to) = transaction.to.clone() {
                addresses.push(to);
            }
        }

        if let Ok(payments) =
            Payment::find_all_by_eth_address(addresses, self.postgres.clone()).wait()
        {
            for (_, payment) in payments.iter().enumerate() {
                for (_, transaction) in block.transactions.iter().enumerate() {
                    if let Some(to) = transaction.to.clone() {
                        if payment.eth_address.clone().unwrap() == to {
                            println!("Publiching payout event for {:?}", transaction.hash);
                            self.redis.do_send(Publish {
                                key: String::from("payout"),
                                value: json!({
                                    "transaction": transaction,
                                    "payment": payment
                                }).to_string(),
                            });
                        }
                    }
                }
            }
        };
    }
}

use std::time::Duration;

use actix::prelude::*;
use actix_web::actix::spawn;
use actix_web::ws::{ClientWriter, Message, ProtocolError};
use futures::{future, Future};
use serde::de::{self, Deserializer, Unexpected};
use serde::Deserialize;
use serde_json;

use consumer::{ConsumerAddr, NewBlock};
use core::app_status::AppStatus;
use core::block::{Block, BlockHeader};
use core::db::postgres::PgExecutorAddr;
use types::U128;

pub struct Subscriber {
    writer: ClientWriter,
    consumer: ConsumerAddr,
    postgres: PgExecutorAddr,
    previous_block_number: Option<U128>,
}

impl Subscriber {
    pub fn new(writer: ClientWriter, postgres: PgExecutorAddr, consumer: ConsumerAddr) -> Self {
        Subscriber {
            writer,
            consumer,
            postgres,
            previous_block_number: None,
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.writer.ping("");
            act.hb(ctx);
        });
    }

    pub fn subscribe_new_blocks(&mut self) {
        println!("Started subscriber");
        let message = json!({
            "id": 1,
            "method": "eth_subscribe",
            "params": ["newHeads"]
        }).to_string();

        self.writer.text(message);
    }
}

impl Actor for Subscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // TODO: Check previously obtained block number from DB,
        // and send `GetBlockByNumber` for each missed blocks.

        ctx.notify(GetBlockNumber);
    }
}

#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    jsonrpc: String,
    id: u32,
    result: String,
}

#[derive(Debug, Deserialize)]
struct Publication {
    jsonrpc: String,
    method: String,
    params: PublicationParams,
}

#[derive(Debug, Deserialize)]
struct PublicationParams {
    subscription: String,
    result: BlockHeader,
}

#[derive(Debug, Deserialize)]
struct BlockResponse {
    jsonrpc: String,
    id: u64,
    result: Block,
}

#[derive(Debug, Deserialize)]
struct BlockNumberResponse {
    jsonrpc: String,
    id: u32,
    #[serde(deserialize_with = "string_to_u128")]
    result: U128,
}

fn string_to_u128<'de, D>(deserializer: D) -> Result<U128, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match i64::from_str_radix(&s[2..], 16) {
        Ok(decimal) => Ok(U128::from_dec_str(&format!("{}", decimal)).unwrap()),
        Err(_) => Err(de::Error::invalid_value(Unexpected::Str(&s), &"U128")),
    }
}

#[derive(Debug, Deserialize)]
struct EmptyResponse {
    jsonrpc: String,
    id: u64,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct GetBlockNumber;

impl Handler<GetBlockNumber> for Subscriber {
    type Result = ();

    fn handle(&mut self, _: GetBlockNumber, _: &mut Context<Self>) -> Self::Result {
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": (),
        }).to_string();

        self.writer.text(message)
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct GetBlockByNumber(pub U128);

impl Handler<GetBlockByNumber> for Subscriber {
    type Result = ();

    fn handle(
        &mut self,
        GetBlockByNumber(block_number): GetBlockByNumber,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let message = json!({
            "jsonrpc": "2.0",
            "id": &block_number.0.low_u64(),
            "method": "eth_getBlockByNumber",
            "params": (format!("{:#x}", &block_number.0), true),
        }).to_string();

        self.writer.text(message)
    }
}

impl StreamHandler<Message, ProtocolError> for Subscriber {
    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        match msg {
            Message::Text(txt) => {
                if let Ok(response) = serde_json::from_str::<BlockNumberResponse>(&txt) {
                    let result = response.result;
                    let address = ctx.address();

                    spawn(
                        AppStatus::find(&self.postgres)
                            .and_then(move |status| match status.block_height {
                                Some(block_height) => {
                                    println!(
                                        "Fetching missed blocks: {} ~ {}",
                                        block_height, result
                                    );

                                    for x in block_height.0.low_u64()..=result.0.low_u64() {
                                        spawn(
                                            address
                                                .send(GetBlockByNumber(U128::from(x)))
                                                .map_err(|_| ()),
                                        )
                                    }

                                    future::ok(())
                                }
                                None => future::ok(()),
                            })
                            .map_err(|_| ()),
                    );

                    self.subscribe_new_blocks();
                    self.hb(ctx);
                    return;
                }

                if let Ok(response) = serde_json::from_str::<SubscriptionResponse>(&txt) {
                    println!("Subscribed: {:?}", response.result);
                    return;
                }

                if let Ok(response) = serde_json::from_str::<BlockResponse>(&txt) {
                    spawn(
                        self.consumer
                            .send(NewBlock(response.result))
                            .map_err(|_| ()),
                    );
                    return;
                }

                if let Ok(notification) = serde_json::from_str::<Publication>(&txt) {
                    let block_number = match notification.params.result.number {
                        Some(number) => number,
                        None => {
                            // TODO: Handle error
                            return;
                        }
                    };

                    if let Some(previous_block_number) = &self.previous_block_number {
                        if &block_number == previous_block_number {
                            return;
                        }
                    }

                    self.previous_block_number = Some(block_number.clone());

                    ctx.run_later(Duration::from_secs(4), move |_, ctx| {
                        spawn(
                            ctx.address()
                                .send(GetBlockByNumber(block_number))
                                .map_err(|_| ()),
                        )
                    });
                    return;
                }

                if let Ok(empty_response) = serde_json::from_str::<EmptyResponse>(&txt) {
                    // Retry request for block with empty response.
                    ctx.run_later(Duration::from_secs(4), move |_, ctx| {
                        spawn(
                            ctx.address()
                                .send(GetBlockByNumber(U128::from(empty_response.id)))
                                .map_err(|_| ()),
                        )
                    });
                    return;
                }
            }
            _ => (),
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

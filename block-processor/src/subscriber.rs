use std::collections::HashMap;
use std::time::Duration;

use actix::prelude::*;
use actix_web::actix::spawn;
use actix_web::ws::{ClientWriter, Message, ProtocolError};
use futures::Future;
use serde_json;

use consumer::{ConsumerAddr, NewBlock, Ready, Startup};
use core::block::{Block, BlockHeader};
use types::U128;

pub struct Subscriber {
    writer: ClientWriter,
    consumer: ConsumerAddr,
    previous_block_number: Option<U128>,
    buffer: HashMap<U128, Block>,
}

impl Subscriber {
    pub fn new(writer: ClientWriter, consumer: ConsumerAddr) -> Self {
        Subscriber {
            writer,
            consumer,
            previous_block_number: None,
            buffer: HashMap::new(),
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.writer.ping("");
            act.hb(ctx);
        });
    }

    pub fn subscribe_new_blocks(&mut self) {
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
        println!("Started subscriber");
        spawn(
            self.consumer
                .send(Startup(ctx.address().recipient()))
                .map_err(|_| ()),
        )
    }
}

impl<'a> Handler<Ready> for Subscriber {
    type Result = ();

    fn handle(&mut self, _: Ready, ctx: &mut Self::Context) -> Self::Result {
        self.hb(ctx);
        self.subscribe_new_blocks();
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
struct EmptyResponse {
    jsonrpc: String,
    id: u64,
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
                if let Ok(response) = serde_json::from_str::<SubscriptionResponse>(&txt) {
                    println!("Subscribed: {:?}", response.result);
                    return;
                }

                if let Ok(response) = serde_json::from_str::<BlockResponse>(&txt) {
                    if let Some(ref previous_block_number) = self.previous_block_number {
                        if previous_block_number.0 + U128::from(1).0
                            != response.result.number.clone().unwrap().0
                        {
                            self.buffer.insert(
                                response.result.number.clone().unwrap(),
                                response.result.clone(),
                            );
                            return;
                        }
                    };

                    self.previous_block_number = response.result.number.clone();

                    spawn(
                        self.consumer
                            .send(NewBlock(response.result.clone()))
                            .map_err(|e| {
                                println!("failed to send {:?}", e);
                                ()
                            }),
                    );

                    let mut incremental = 1;
                    while let Some(buffered) = self.buffer.get(&U128::from(
                        response.result.number.clone().unwrap().0.low_u64() + incremental,
                    )) {
                        self.previous_block_number = buffered.number.clone();

                        spawn(
                            self.consumer
                                .send(NewBlock((*buffered).clone()))
                                .map_err(|e| {
                                    println!("failed to send {:?}", e);
                                    ()
                                }),
                        );
                        incremental += 1;
                    }

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
                            // Already processed block.
                            return;
                        }
                    }

                    ctx.run_later(Duration::from_secs(4), move |_, ctx| {
                        ctx.notify(GetBlockByNumber(block_number))
                    });
                    return;
                }

                if let Ok(empty_response) = serde_json::from_str::<EmptyResponse>(&txt) {
                    // Retry request for block with empty response.
                    ctx.run_later(Duration::from_secs(4), move |_, ctx| {
                        ctx.notify(GetBlockByNumber(U128::from(empty_response.id)))
                    });
                    return;
                }
            }
            _ => (),
        }
    }

    fn started(&mut self, _: &mut Context<Self>) {}

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}

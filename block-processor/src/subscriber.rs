use std::time::Duration;

use actix::prelude::*;
use actix_web::actix::spawn;
use actix_web::ws::{ClientWriter, Message, ProtocolError};
use futures::Future;
use serde_json;

use consumer::{ConsumerAddr, NewBlock};
use core::block::{Block, BlockHeader};

pub struct Subscriber {
    writer: ClientWriter,
    consumer: ConsumerAddr,
    previous_block_number: Option<u128>,
}

impl Subscriber {
    pub fn new(writer: ClientWriter, consumer: ConsumerAddr) -> Self {
        Subscriber {
            writer,
            consumer,
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
        self.subscribe_new_blocks();
        self.hb(ctx)
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
    id: u32,
    result: Block,
}

#[derive(Debug, Deserialize)]
struct EmptyResponse {
    jsonrpc: String,
    id: u32,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct GetBlockByNumber(pub u128);

impl Handler<GetBlockByNumber> for Subscriber {
    type Result = ();

    fn handle(
        &mut self,
        GetBlockByNumber(block_number): GetBlockByNumber,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let message = json!({
            "jsonrpc": "2.0",
            "id": &block_number,
            "method": "eth_getBlockByNumber",
            "params": (format!("{:#x}", &block_number), true),
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
                                .send(GetBlockByNumber(empty_response.id as u128))
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

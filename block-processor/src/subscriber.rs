use std::collections::HashMap;
use std::time::Duration;

use actix::fut::wrap_future;
use actix::prelude::*;
use actix_web::ws::{ClientWriter, Message, ProtocolError};
use futures::{future, Future};
use serde_json;

use consumer::{ConsumerAddr, ProcessBlock, Startup};
use core::block::{Block, BlockHeader};
use types::U128;

use errors::Error;

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

        ctx.spawn(
            wrap_future(
                self.consumer
                    .send(Startup)
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e))),
            ).from_err::<Error>()
                .and_then(|current_block_number, _, ctx: &mut Context<Self>| {
                    wrap_future(
                        ctx.address()
                            .send(Ready(current_block_number))
                            .from_err()
                            .and_then(|res| res.map_err(|e| Error::from(e))),
                    )
                })
                .map_err(|e, _, _| match e {
                    Error::ModelError(err) => println!("Model error: {}", err),
                    Error::MailboxError(err) => println!("Mailbox error: {}", err),
                    _ => println!(""),
                })
                .map(|_, _, _| ()),
        );
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Ready(pub U128);

impl<'a> Handler<Ready> for Subscriber {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        Ready(current_block_number): Ready,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.previous_block_number = Some(current_block_number);
        self.hb(ctx);
        self.subscribe_new_blocks();
        Box::new(future::ok(()))
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
    id: U128,
    result: Block,
}

#[derive(Debug, Deserialize)]
struct EmptyResponse {
    jsonrpc: String,
    id: U128,
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
            "id": format!("{}", block_number),
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
                        if previous_block_number.clone() + U128::from(1)
                            > response.result.number.clone().unwrap()
                        {
                            return;
                        }

                        if previous_block_number.clone() + U128::from(1)
                            != response.result.number.clone().unwrap()
                        {
                            self.buffer.insert(
                                response.result.number.clone().unwrap(),
                                response.result.clone(),
                            );
                            return;
                        }
                    }

                    self.previous_block_number = response.result.number.clone();

                    ctx.spawn(wrap_future(
                        self.consumer
                            .send(ProcessBlock(response.result.clone()))
                            .from_err::<Error>()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                            .map_err(|e| {
                                println!("failed to send {:?}", e);
                                ()
                            }),
                    ));

                    let mut remove_from_buffer = Vec::<U128>::new();
                    let mut incremental = 1;

                    while let Some(buffered) = self
                        .buffer
                        .get(&(response.result.number.clone().unwrap() + U128::from(incremental)))
                    {
                        self.previous_block_number = buffered.number.clone();

                        ctx.spawn(wrap_future(
                            self.consumer
                                .send(ProcessBlock((*buffered).clone()))
                                .from_err::<Error>()
                                .and_then(|res| res.map_err(|e| Error::from(e)))
                                .map_err(|e| {
                                    println!("failed to send {:?}", e);
                                    ()
                                }),
                        ));

                        remove_from_buffer.push(
                            response.result.number.clone().unwrap() + U128::from(incremental),
                        );

                        incremental += 1;
                    }

                    for (_, ref x) in remove_from_buffer.iter().enumerate() {
                        self.buffer.remove(x);
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

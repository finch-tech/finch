use std::time::Duration;

use actix::fut::wrap_future;
use actix::prelude::*;
use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use futures::{future, stream, Future, Stream};
use futures_timer::Delay;

use core::app_status::AppStatus;
use core::db::postgres::PgExecutorAddr;
use errors::Error;
use ethereum_client::{Client, Error as EthereumClientError};
use processor::{ProcessBlock, ProcessorAddr};
use types::U128;

pub struct Poller {
    processor: ProcessorAddr,
    postgres: PgExecutorAddr,
    ethereum_rpc_client: Client,
    skip_missed_blocks: bool,
}

impl Poller {
    pub fn new(
        processor: ProcessorAddr,
        postgres: PgExecutorAddr,
        ethereum_rpc_client: Client,
        skip_missed_blocks: bool,
    ) -> Self {
        Poller {
            processor,
            postgres,
            ethereum_rpc_client,
            skip_missed_blocks,
        }
    }
}

impl Actor for Poller {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let address = ctx.address();

        let poller_process = address
            .send(ProcessMissedBlocks)
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
            .and_then(move |current_block| {
                address
                    .send(Poll(current_block))
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
            })
            .map_err(|e| match e {
                _ => println!("Poller error: {:?}", e),
            });

        ctx.spawn(wrap_future(poller_process));
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct ProcessMissedBlocks;

impl<'a> Handler<ProcessMissedBlocks> for Poller {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(
        &mut self,
        ProcessMissedBlocks: ProcessMissedBlocks,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let processor = self.processor.clone();
        let eth_client = self.ethereum_rpc_client.clone();
        let skip_missed_blocks = self.skip_missed_blocks;

        let app_status = AppStatus::find(&self.postgres).from_err::<Error>();
        let current_block_number = eth_client.get_block_number().from_err::<Error>();

        Box::new(
            app_status
                .join(current_block_number)
                .from_err::<Error>()
                .and_then(move |(status, current_block_number)| {
                    if skip_missed_blocks {
                        return future::Either::A(future::ok(current_block_number));
                    }

                    if let Some(block_height) = status.eth_block_height {
                        if block_height == current_block_number {
                            return future::Either::A(future::ok(block_height));
                        }

                        println!(
                            "Fetching missed blocks: {} ~ {}",
                            block_height + U128::from(1),
                            current_block_number
                        );

                        future::Either::B(
                            stream::unfold(block_height + U128::from(1), move |block_number| {
                                if block_number <= current_block_number {
                                    let next_block_number = block_number.clone() + U128::from(1);
                                    Some(future::ok::<_, _>((block_number, next_block_number)))
                                } else {
                                    None
                                }
                            }).for_each(move |block_number| {
                                let processor = processor.clone();

                                eth_client
                                    .get_block_by_number(U128::from(block_number))
                                    .from_err()
                                    .and_then(move |block| {
                                        processor
                                            .send(ProcessBlock(block))
                                            .from_err()
                                            .and_then(|res| res.map_err(|e| Error::from(e)))
                                    })
                            })
                                .and_then(move |_| {
                                    address
                                        .send(ProcessMissedBlocks)
                                        .from_err()
                                        .and_then(|res| res.map_err(|e| Error::from(e)))
                                }),
                        )
                    } else {
                        return future::Either::A(future::ok(current_block_number));
                    }
                }),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Poll(pub U128);

impl<'a> Handler<Poll> for Poller {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, Poll(block_number): Poll, ctx: &mut Self::Context) -> Self::Result {
        let address = ctx.address();
        let processor = self.processor.clone();
        let eth_client = self.ethereum_rpc_client.clone();

        let polling = eth_client
            .get_block_by_number(block_number.clone() + U128::from(1))
            .from_err::<Error>()
            .and_then(move |block| {
                processor
                    .send(ProcessBlock(block))
                    .from_err::<Error>()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
                    .map(move |_| block_number + U128::from(1))
            })
            .or_else(move |e| match e {
                Error::EthereumClientError(e) => match e {
                    EthereumClientError::EmptyResponseError => future::ok(block_number.clone()),
                    EthereumClientError::SendRequestError(e) => match e {
                        SendRequestError::Timeout => future::ok(block_number.clone()),
                        _ => future::err(Error::EthereumClientError(
                            EthereumClientError::SendRequestError(e),
                        )),
                    },
                    EthereumClientError::PayloadError(e) => match e {
                        PayloadError::Io(_) => future::ok(block_number.clone()),
                        _ => future::err(Error::EthereumClientError(
                            EthereumClientError::PayloadError(e),
                        )),
                    },
                    _ => future::err(Error::from(e)),
                },

                Error::ModelError(e) => future::err(Error::from(e)),
                Error::MailboxError(e) => future::err(Error::from(e)),
                Error::IoError(e) => future::err(Error::from(e)),
            })
            .and_then(move |block_number| {
                Delay::new(Duration::from_secs(3))
                    .from_err::<Error>()
                    .and_then(move |_| {
                        address
                            .send(Poll(block_number.clone()))
                            .from_err::<Error>()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                    })
            })
            .map(|_| ());

        Box::new(polling)
    }
}

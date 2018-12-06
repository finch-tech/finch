use std::time::Duration;

use actix::{fut::wrap_future, prelude::*};
use futures::{future, stream, Future, Stream};
use futures_timer::Delay;

use bitcoin::{
    processor::{ProcessBlock, ProcessorAddr},
    Error,
};
use core::{app_status::AppStatus, bitcoin::Transaction, db::postgres::PgExecutorAddr};
use rpc_client::{bitcoin::RpcClient, errors::Error as RpcClientError};
use types::U128;

const RETRY_LIMIT: i8 = 10;

pub struct Poller {
    processor: ProcessorAddr,
    postgres: PgExecutorAddr,
    rpc_client: RpcClient,
    skip_missed_blocks: bool,
}

impl Poller {
    pub fn new(
        processor: ProcessorAddr,
        postgres: PgExecutorAddr,
        rpc_client: RpcClient,
        skip_missed_blocks: bool,
    ) -> Self {
        Poller {
            processor,
            postgres,
            rpc_client,
            skip_missed_blocks,
        }
    }
}

impl Actor for Poller {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Start;

impl Handler<Start> for Poller {
    type Result = ();

    fn handle(&mut self, Start: Start, ctx: &mut Self::Context) -> Self::Result {
        let address = ctx.address();

        let poller_process = address
            .send(ProcessMissedBlocks)
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
            .and_then(move |next_block| {
                address
                    .send(Poll {
                        block_number: next_block,
                        retry_count: 0,
                    })
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
            })
            .map(|_| ())
            .map_err(|e| match e {
                _ => println!("Poller error: {:?}", e),
            });

        ctx.spawn(wrap_future(poller_process));
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct ProcessMissedBlocks;

impl Handler<ProcessMissedBlocks> for Poller {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(
        &mut self,
        ProcessMissedBlocks: ProcessMissedBlocks,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let processor = self.processor.clone();
        let rpc_client = self.rpc_client.clone();
        let skip_missed_blocks = self.skip_missed_blocks;

        let app_status = AppStatus::find(&self.postgres).from_err::<Error>();
        let current_block_number = rpc_client.get_block_count().from_err::<Error>();

        Box::new(
            app_status
                .join(current_block_number)
                .from_err::<Error>()
                .and_then(move |(status, current_block_number)| {
                    if skip_missed_blocks {
                        return future::Either::A(future::ok(current_block_number));
                    }

                    if let Some(block_height) = status.btc_block_height {
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
                                    let next_block_number = block_number + U128::from(1);
                                    Some(future::ok::<_, _>((block_number, next_block_number)))
                                } else {
                                    None
                                }
                            })
                            .for_each(move |block_number| {
                                let processor = processor.clone();
                                let rpc_client = rpc_client.clone();
                                let rpc_client_b = rpc_client.clone();

                                rpc_client
                                    .get_block_hash(block_number)
                                    .from_err::<Error>()
                                    .and_then(move |hash| {
                                        rpc_client.get_block(hash).from_err::<Error>()
                                    })
                                    .and_then(move |block| {
                                        future::ok(stream::iter_ok(
                                            block.tx_hashes[1..].to_vec().clone(),
                                        ))
                                        .flatten_stream()
                                        .and_then(move |hash| {
                                            rpc_client_b.get_raw_transaction(hash).from_err()
                                        })
                                        .fold(
                                            Vec::new(),
                                            |mut vec,
                                             tx|
                                             -> Box<
                                                Future<Item = Vec<Transaction>, Error = Error>,
                                            > {
                                                vec.push(tx);
                                                Box::new(future::ok(vec))
                                            },
                                        )
                                        .and_then(
                                            move |transactions| {
                                                let mut block = block;
                                                block.transactions = Some(transactions);

                                                processor
                                                    .send(ProcessBlock(block))
                                                    .from_err()
                                                    .and_then(|res| res.map_err(|e| Error::from(e)))
                                            },
                                        )
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
pub struct Poll {
    pub block_number: U128,
    pub retry_count: i8,
}

impl Handler<Poll> for Poller {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        Poll {
            block_number,
            retry_count,
        }: Poll,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let processor = self.processor.clone();
        let rpc_client = self.rpc_client.clone();
        let rpc_client_b = self.rpc_client.clone();

        if retry_count == RETRY_LIMIT {
            return Box::new(future::err(Error::RetryLimitError(retry_count)));
        }

        let polling = rpc_client
            .get_block_hash(block_number)
            .from_err::<Error>()
            .and_then(move |hash| rpc_client.get_block(hash).from_err::<Error>())
            .and_then(move |block| {
                let mut requests = Vec::new();

                for idx in 1..block.tx_hashes.len() {
                    let hash = block.tx_hashes[idx];
                    requests.push(rpc_client_b.get_raw_transaction(hash).from_err())
                }

                future::join_all(requests).and_then(move |transactions| {
                    let mut block = block;
                    block.transactions = Some(transactions);

                    processor
                        .send(ProcessBlock(block))
                        .from_err()
                        .and_then(|res| res.map_err(|e| Error::from(e)))
                        .map(move |_| (block_number + U128::from(1), 0))
                })
            })
            .or_else(move |e| match e {
                Error::RpcClientError(e) => match e {
                    RpcClientError::EmptyResponseError => future::ok((block_number, 0)),
                    _ => future::ok((block_number, retry_count + 1)),
                },
                _ => future::err(e),
            })
            .and_then(move |(block_number, retry_count)| {
                Delay::new(Duration::from_secs(3))
                    .from_err::<Error>()
                    .and_then(move |_| {
                        address
                            .send(Poll {
                                block_number: block_number,
                                retry_count,
                            })
                            .from_err::<Error>()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                    })
            })
            .map(|_| ());

        Box::new(polling)
    }
}

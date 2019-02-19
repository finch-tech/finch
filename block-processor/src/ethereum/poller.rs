use std::time::Duration;

use actix::prelude::*;
use futures::{future, stream, Future, Stream};
use futures_timer::Delay;

use blockchain_api_client::{
    errors::Error as BlockchainApiClientError,
    ethereum::{BlockchainApiClientAddr, GetBlockByNumber, GetBlockNumber},
};
use core::{
    db::postgres::PgExecutorAddr,
    ethereum::{BlockchainStatus, BlockchainStatusPayload},
};
use ethereum::{
    errors::Error,
    processor::{ProcessBlock, ProcessorAddr},
};
use types::{ethereum::Network, U128};

const RETRY_LIMIT: usize = 10;

pub struct Poller {
    processor: ProcessorAddr,
    postgres: PgExecutorAddr,
    blockchain_api_client: BlockchainApiClientAddr,
    network: Network,
}

impl Poller {
    pub fn new(
        processor: ProcessorAddr,
        postgres: PgExecutorAddr,
        blockchain_api_client: BlockchainApiClientAddr,
        network: Network,
    ) -> Self {
        Poller {
            processor,
            postgres,
            blockchain_api_client,
            network,
        }
    }
}

impl Actor for Poller {
    type Context = Context<Self>;
}

impl Supervised for Poller {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        match ctx.address().try_send(StartPolling {
            skip_missed_blocks: false,
        }) {
            Ok(_) => info!("Restarting"),
            Err(_) => error!("Failed to start polling on restart"),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stop;

impl Handler<Stop> for Poller {
    type Result = ();

    fn handle(&mut self, _: Stop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct StartPolling {
    pub skip_missed_blocks: bool,
}

impl Handler<StartPolling> for Poller {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        StartPolling { skip_missed_blocks }: StartPolling,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let _address = ctx.address();

        let process = address
            .send(Bootstrap { skip_missed_blocks })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
            .and_then(move |current_block| {
                address
                    .send(Poll {
                        block_number: current_block + U128::from(1),
                        retry_count: 0,
                    })
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
            })
            .or_else(move |e| {
                error!("{:?}", e);
                _address.send(Stop).from_err()
            })
            .map(|_| ());

        Box::new(process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct Bootstrap {
    pub skip_missed_blocks: bool,
}

impl Handler<Bootstrap> for Poller {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(
        &mut self,
        Bootstrap { skip_missed_blocks }: Bootstrap,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();
        let processor = self.processor.clone();
        let blockchain_api_client = self.blockchain_api_client.clone();
        let postgres = self.postgres.clone();
        let network = self.network;

        let bootstrap_process = blockchain_api_client
            .send(GetBlockNumber)
            .from_err::<Error>()
            .and_then(move |res| res.map_err(|e| Error::from(e)))
            .and_then(move |current_block_number| {
                BlockchainStatus::find(network, &postgres)
                    .from_err()
                    .or_else(
                        move |e| -> Box<Future<Item = BlockchainStatus, Error = Error>> {
                            match e {
                                Error::ModelError(_) => {
                                    let payload = BlockchainStatusPayload {
                                        network: Some(network),
                                        block_height: Some(current_block_number),
                                    };

                                    Box::new(
                                        BlockchainStatus::insert(payload, &postgres).from_err(),
                                    )
                                }
                                _ => Box::new(future::err(e)),
                            }
                        },
                    )
                    .map(move |status| (status, current_block_number))
            })
            .from_err::<Error>()
            .and_then(move |(status, current_block_number)| {
                let block_height = status.block_height;

                if skip_missed_blocks {
                    return future::Either::A(future::ok(current_block_number));
                }

                if block_height == current_block_number {
                    return future::Either::A(future::ok(block_height));
                }

                info!(
                    "Fetching missed blocks: {} ~ {}",
                    block_height + U128::from(1),
                    current_block_number
                );

                future::Either::B(
                    stream::unfold(block_height + U128::from(1), move |block_number| {
                        if block_number <= current_block_number {
                            return Some(future::ok((block_number, block_number + U128::from(1))));
                        }

                        None
                    })
                    .for_each(move |block_number| {
                        let processor = processor.clone();

                        blockchain_api_client
                            .send(GetBlockByNumber(block_number))
                            .from_err()
                            .and_then(move |res| res.map_err(|e| Error::from(e)))
                            .and_then(move |block| {
                                processor
                                    .send(ProcessBlock(block))
                                    .from_err()
                                    .and_then(|res| res.map_err(|e| Error::from(e)))
                            })
                    })
                    .and_then(move |_| {
                        address
                            .send(Bootstrap { skip_missed_blocks })
                            .from_err()
                            .and_then(|res| res.map_err(|e| Error::from(e)))
                    }),
                )
            });

        Box::new(bootstrap_process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Poll {
    pub block_number: U128,
    pub retry_count: usize,
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
        let blockchain_api_client = self.blockchain_api_client.clone();

        if retry_count == RETRY_LIMIT {
            return Box::new(future::err(Error::RetryLimitError(retry_count)));
        }

        let polling = blockchain_api_client
            .send(GetBlockByNumber(block_number))
            .from_err::<Error>()
            .and_then(move |res| res.map_err(|e| Error::from(e)))
            .from_err::<Error>()
            .and_then(move |block| {
                processor
                    .send(ProcessBlock(block))
                    .from_err::<Error>()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
                    .map(move |_| (block_number + U128::from(1), 0))
            })
            .or_else(move |e| match e {
                Error::BlockchainApiClientError(e) => match e {
                    BlockchainApiClientError::EmptyResponseError => future::ok((block_number, 0)),
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

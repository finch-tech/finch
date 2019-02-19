use std::{collections::HashSet, iter::FromIterator, time::Duration};

use actix::prelude::*;
use futures::{future, Future};
use futures_timer::Delay;

use blockchain_api_client::{
    errors::Error as BlockchainApiClientError,
    ethereum::{GetPendingBlock, BlockchainApiClientAddr},
};
use core::ethereum::Transaction;
use ethereum::{
    errors::Error,
    processor::{ProcessPendingTransactions, ProcessorAddr},
};

const RETRY_LIMIT: usize = 10;

pub struct Poller {
    processor: ProcessorAddr,
    blockchain_api_client: BlockchainApiClientAddr,
}

impl Poller {
    pub fn new(processor: ProcessorAddr, blockchain_api_client: BlockchainApiClientAddr) -> Self {
        Poller {
            processor,
            blockchain_api_client,
        }
    }
}

impl Actor for Poller {
    type Context = Context<Self>;
}

impl Supervised for Poller {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        match ctx.address().try_send(StartPolling) {
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
pub struct StartPolling;

impl Handler<StartPolling> for Poller {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, _: StartPolling, ctx: &mut Self::Context) -> Self::Result {
        let address = ctx.address();

        let process = ctx
            .address()
            .send(Poll {
                previous: HashSet::new(),
                retry_count: 0,
            })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
            .or_else(move |e| {
                error!("{:?}", e);
                address.send(Stop).from_err()
            })
            .map(|_| ());

        Box::new(process)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Poll {
    pub previous: HashSet<Transaction>,
    pub retry_count: usize,
}

impl Handler<Poll> for Poller {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        Poll {
            previous,
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
            .send(GetPendingBlock)
            .from_err::<Error>()
            .and_then(move |res| res.map_err(|e| Error::from(e)))
            .and_then(move |block| {
                let transactions = HashSet::from_iter(block.transactions.iter().cloned());

                let diff = transactions
                    .clone()
                    .iter()
                    .filter(|ref t| !previous.contains(t))
                    .map(|t| t.to_owned())
                    .collect::<Vec<Transaction>>();

                processor
                    .send(ProcessPendingTransactions(diff))
                    .from_err::<Error>()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
                    .map(move |_| 0)
                    .or_else(move |e| match e {
                        Error::BlockchainApiClientError(e) => match e {
                            BlockchainApiClientError::EmptyResponseError => future::ok(0),
                            _ => future::ok(retry_count + 1),
                        },
                        _ => future::err(e),
                    })
                    .and_then(move |retry_count| {
                        Delay::new(Duration::from_secs(3))
                            .from_err::<Error>()
                            .and_then(move |_| {
                                address
                                    .send(Poll {
                                        previous: transactions,
                                        retry_count,
                                    })
                                    .from_err::<Error>()
                                    .and_then(|res| res.map_err(|e| Error::from(e)))
                            })
                    })
                    .map(|_| ())
            });

        Box::new(polling)
    }
}

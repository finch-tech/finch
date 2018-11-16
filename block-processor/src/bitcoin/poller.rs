use actix::prelude::*;

use bitcoin::processor::ProcessorAddr;
use core::db::postgres::PgExecutorAddr;
use rpc_client::bitcoin::RpcClient;

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

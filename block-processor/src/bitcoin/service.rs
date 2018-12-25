use actix::prelude::*;

use bitcoin::{
    poller::{Poller, StartPolling},
    processor::Processor,
};
use core::db::postgres;
use rpc_client::bitcoin::RpcClient;

pub fn run(postgres: postgres::PgExecutorAddr, rpc_client: RpcClient, skip_missed_blocks: bool) {
    let pg = postgres.clone();
    let block_processor = Arbiter::start(move |_| Processor { postgres: pg });

    let poller = Arbiter::start(move |_| Poller::new(block_processor, postgres, rpc_client));

    poller.do_send(StartPolling { skip_missed_blocks });
}

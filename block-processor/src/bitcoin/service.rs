use actix::prelude::*;

use bitcoin::{
    poller::{Poller, StartPolling},
    processor::Processor,
};
use core::db::postgres;
use rpc_client::bitcoin::RpcClient;
use types::bitcoin::Network;

pub fn run(
    postgres: postgres::PgExecutorAddr,
    rpc_client: RpcClient,
    network: Network,
    skip_missed_blocks: bool,
) {
    let pg = postgres.clone();
    let block_processor = Arbiter::start(move |_| Processor {
        network,
        postgres: pg,
    });

    let poller =
        Supervisor::start(move |_| Poller::new(block_processor, postgres, rpc_client, network));

    poller.do_send(StartPolling { skip_missed_blocks });
}

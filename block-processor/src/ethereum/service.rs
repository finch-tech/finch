use actix::prelude::*;

use core::db::postgres;
use ethereum::{
    poller::{Poller, StartPolling},
    processor::Processor,
};
use rpc_client::ethereum::RpcClient;
use types::ethereum::Network;

pub fn run(
    postgres: postgres::PgExecutorAddr,
    rpc_client: RpcClient,
    network: Network,
    skip_missed_blocks: bool,
) {
    let pg = postgres.clone();
    let block_processor_address = Arbiter::start(move |_| Processor {
        network,
        postgres: pg,
    });

    let poller = Supervisor::start(move |_| {
        Poller::new(block_processor_address, postgres, rpc_client, network)
    });

    poller.do_send(StartPolling { skip_missed_blocks });
}

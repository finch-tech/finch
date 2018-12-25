use actix::prelude::*;

use core::db::postgres;
use ethereum::{
    poller::{Poller, StartPolling},
    processor::Processor,
};
use rpc_client::ethereum::RpcClient;

pub fn run(
    postgres: postgres::PgExecutorAddr,
    rpc_client: RpcClient,
    skip_missed_blocks: bool,
) {
    let pg = postgres.clone();
    let block_processor_address = Arbiter::start(move |_| Processor { postgres: pg });

    let poller =
        Arbiter::start(move |_| Poller::new(block_processor_address, postgres, rpc_client));

    poller.do_send(StartPolling { skip_missed_blocks });
}

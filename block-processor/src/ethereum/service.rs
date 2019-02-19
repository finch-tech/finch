use actix::prelude::*;

use core::db::postgres;
use ethereum::{
    pb_poller::{Poller as PendingBlocksPoller, StartPolling as StartPollingPendings},
    poller::{Poller, StartPolling},
    processor::Processor,
};
use rpc_client::ethereum::RpcClientAddr;
use types::ethereum::Network;

pub fn run(
    postgres: postgres::PgExecutorAddr,
    rpc_client: RpcClientAddr,
    network: Network,
    skip_missed_blocks: bool,
) -> (Addr<Processor>, Addr<Poller>, Addr<PendingBlocksPoller>) {
    let pg = postgres.clone();
    let block_processor = Arbiter::start(move |_| Processor {
        network,
        postgres: pg,
    });

    let _block_processor = block_processor.clone();
    let _postgres = postgres.clone();
    let _rpc_client = rpc_client.clone();
    let poller =
        Supervisor::start(move |_| Poller::new(_block_processor, _postgres, _rpc_client, network));
    poller.do_send(StartPolling { skip_missed_blocks });

    let _block_processor = block_processor.clone();
    let pb_poller =
        Supervisor::start(move |_| PendingBlocksPoller::new(_block_processor, rpc_client));
    pb_poller.do_send(StartPollingPendings);

    (block_processor, poller, pb_poller)
}

use actix::prelude::*;

use bitcoin::{
    pb_poller::{Poller as PendingBlocksPoller, StartPolling as StartPollingPendings},
    poller::{Poller, StartPolling},
    processor::Processor,
};
use blockchain_api_client::bitcoin::BlockchainApiClientAddr;
use core::db::postgres;
use types::bitcoin::Network;

pub fn run(
    postgres: postgres::PgExecutorAddr,
    blockchain_api_client: BlockchainApiClientAddr,
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
    let _blockchain_api_client = blockchain_api_client.clone();
    let poller = Supervisor::start(move |_| {
        Poller::new(_block_processor, _postgres, _blockchain_api_client, network)
    });
    poller.do_send(StartPolling { skip_missed_blocks });

    let _block_processor = block_processor.clone();
    let pb_poller = Supervisor::start(move |_| {
        PendingBlocksPoller::new(_block_processor, blockchain_api_client)
    });
    pb_poller.do_send(StartPollingPendings);

    (block_processor, poller, pb_poller)
}

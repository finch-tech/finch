use actix::prelude::*;

use core::db::postgres;
use ethereum::poller::Poller;
use ethereum::processor::Processor;
use rpc_client::ethereum::RpcClient;

pub fn run(postgres_url: String, rpc_client: RpcClient, skip_missed_blocks: bool) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let pg_processor = pg_addr.clone();
        let block_processor_address = Arbiter::start(move |_| Processor {
            postgres: pg_processor,
        });

        Arbiter::start(move |_| {
            Poller::new(
                block_processor_address,
                pg_addr,
                rpc_client,
                skip_missed_blocks,
            )
        });
    });
}

use actix::prelude::*;

use core::db::postgres;
use ethereum_client::Client as EthClient;
use poller::Poller;
use processor::Processor;

pub fn run(postgres_url: String, ethereum_rpc_url: String, skip_missed_blocks: bool) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let pg_processor = pg_addr.clone();
        let processor_address = Arbiter::start(move |_| Processor {
            postgres: pg_processor,
        });

        Arbiter::start(move |_| {
            Poller::new(
                processor_address,
                pg_addr,
                EthClient::new(ethereum_rpc_url),
                skip_missed_blocks,
            )
        });
    });
}

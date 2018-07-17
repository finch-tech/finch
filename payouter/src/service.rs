use actix::prelude::*;

use core::db::{postgres, redis};
use payouter::Payouter;

pub fn run(postgres_url: String, redis_url: String, ethereum_rpc_url: String, chain_id: u64) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let subscriber_addr = Arbiter::start(move |_| redis::RedisSubscriber::new(&redis_url));

        Arbiter::start(move |_| {
            Payouter::new(pg_addr, subscriber_addr, ethereum_rpc_url, chain_id)
        });
    });
}

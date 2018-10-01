use actix::prelude::*;
use actix_web::actix::spawn;

use core::db::{postgres};
use monitor::Monitor;
use payouter::Payouter;

pub fn run(postgres_url: String: String, ethereum_rpc_url: String, chain_id: u64) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let pg_payouter = pg_addr.clone();
        let payouter_addr = Arbiter::start(move |_| {
            Payouter::new(pg_payouter, ethereum_rpc_url, chain_id)
        });

        Arbiter::start(move |_| Monitor::new(payouter_addr.clone(), pg_addr.clone()));
    });
}

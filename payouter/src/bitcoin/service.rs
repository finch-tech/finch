use actix::prelude::*;

use super::{monitor::Monitor, payouter::Payouter};
use core::db::postgres;
use rpc_client::bitcoin::RpcClient;
use types::bitcoin::Network as BtcNetwork;

pub fn run(postgres_url: String, rpc_client: RpcClient, network: BtcNetwork) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let pg_payouter = pg_addr.clone();
        let payouter = Arbiter::start(move |_| Payouter::new(pg_payouter, rpc_client, network));

        Arbiter::start(move |_| Monitor::new(payouter, pg_addr));
    });
}

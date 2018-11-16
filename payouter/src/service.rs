use actix::prelude::*;

use core::db::postgres;
use monitor::Monitor;
use payouter::Payouter;
use rpc_client::ethereum::RpcClient as EthRpcClient;
use types::{BtcNetwork, EthNetwork};

pub fn run(
    postgres_url: String,
    eth_rpc_client: EthRpcClient,
    eth_network: EthNetwork,
    btc_network: BtcNetwork,
) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let pg_payouter = pg_addr.clone();
        let payouter_addr = Arbiter::start(move |_| {
            Payouter::new(pg_payouter, eth_rpc_client, eth_network, btc_network)
        });

        Arbiter::start(move |_| Monitor::new(payouter_addr, pg_addr));
    });
}

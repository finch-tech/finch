use actix::prelude::*;

use super::{monitor::Monitor, payouter::Payouter};
use core::db::postgres;
use rpc_client::bitcoin::RpcClient;
use types::bitcoin::Network as BtcNetwork;

pub fn run(postgres: postgres::PgExecutorAddr, rpc_client: RpcClient, network: BtcNetwork) {
    let pg = postgres.clone();
    let payouter = Arbiter::start(move |_| Payouter::new(pg, rpc_client, network));

    Arbiter::start(move |_| Monitor::new(payouter, postgres));
}

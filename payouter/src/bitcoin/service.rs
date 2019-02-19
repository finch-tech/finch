use actix::prelude::*;

use super::{monitor::Monitor, payouter::Payouter};
use core::db::postgres;
use blockchain_api_client::bitcoin::BlockchainApiClientAddr;
use types::bitcoin::Network as BtcNetwork;

pub fn run(postgres: postgres::PgExecutorAddr, blockchain_api_client: BlockchainApiClientAddr, network: BtcNetwork) {
    let pg = postgres.clone();
    let payouter = Arbiter::start(move |_| Payouter::new(pg, blockchain_api_client, network));

    Arbiter::start(move |_| Monitor::new(payouter, network, postgres));
}

use actix::prelude::*;

use super::{monitor::Monitor, payouter::Payouter};
use core::db::postgres;
use blockchain_api_client::ethereum::BlockchainApiClientAddr;
use types::ethereum::Network as EthNetwork;

pub fn run(postgres: postgres::PgExecutorAddr, blockchain_api_client: BlockchainApiClientAddr, network: EthNetwork) {
    let pg = postgres.clone();
    let payouter = Arbiter::start(move |_| Payouter::new(pg, blockchain_api_client, network));

    Arbiter::start(move |_| Monitor::new(payouter, network, postgres));
}

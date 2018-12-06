extern crate dotenv;
extern crate env_logger;

extern crate config;
extern crate payouter;

use std::env;

use config::Config;
use payouter::bitcoin::service;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    let config = Config::new();

    service::run(
        config.postgres_url,
        config.btc_rpc_client,
        config.btc_network,
    );
}

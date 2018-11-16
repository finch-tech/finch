extern crate env_logger;

extern crate block_processor;
extern crate config;

use std::env;

use block_processor::ethereum::service;
use config::Config;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let mut skip_missed_blocks = false;
    if let Ok(flag) = env::var("SKIP_MISSED_BLOCKS") {
        if flag == "1" {
            skip_missed_blocks = true;
        }
    };

    let config = Config::new();

    service::run(
        config.postgres_url,
        config.eth_rpc_client,
        skip_missed_blocks,
    );
}

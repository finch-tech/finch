extern crate dotenv;
extern crate env_logger;

extern crate block_processor;

use std::env;

use block_processor::service;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    let mut skip_missed_blocks = false;
    if let Ok(_) = env::var("SKIP_MISSED_BLOCKS") {
        skip_missed_blocks = true;
    };

    let postgres_url =
        env::var("POSTGRES_URL").expect("POSTGRES_URL environment variable must be set.");
    let ethereum_ws_url =
        env::var("ETHEREUM_WS_URL").expect("ETHEREUM_WS_URL environment variable must be set.");
    let ethereum_rpc_url =
        env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL environment variable must be set.");

    service::run(
        postgres_url,
        ethereum_ws_url,
        ethereum_rpc_url,
        skip_missed_blocks,
    );
}

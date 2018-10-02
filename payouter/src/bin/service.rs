extern crate dotenv;
extern crate env_logger;

extern crate payouter;

use std::env;

use payouter::service;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    let postgres_url =
        env::var("POSTGRES_URL").expect("POSTGRES_URL environment variable must be set.");
    let ethereum_rpc_url =
        env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL environment variable must be set.");
    let chain_id = env::var("CHAIN_ID").expect("CHAIN_ID environment variable must be set.");
    let chain_id = chain_id.parse::<u64>().unwrap();

    service::run(postgres_url, ethereum_rpc_url, chain_id);
}

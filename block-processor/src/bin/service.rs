extern crate dotenv;
extern crate env_logger;

extern crate block_processor;

use std::env;

use block_processor::service;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    let postgres_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set.");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL environment variable must be set.");
    let ethereum_ws_url =
        env::var("ETHEREUM_WS_URL").expect("ETHEREUM_WS_URL environment variable must be set.");

    service::run(postgres_url, redis_url, ethereum_ws_url);
}

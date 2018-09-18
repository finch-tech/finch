extern crate dotenv;
extern crate env_logger;
extern crate server;

use server::server as web_server;
use std::env;

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    let postgres_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set.");
    let private_key_path =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set.");
    let public_key_path =
        env::var("PUBLIC_KEY").expect("PUBLIC_KEY environment variable must be set.");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL environment variable must be set.");
    let ethereum_rpc_url =
        env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL environment variable must be set.");

    let host = env::var("HOST").expect("HOST environment variable must be set.");
    let port = env::var("PORT").expect("PORT environment variable must be set.");

    web_server::run(
        host,
        port,
        private_key_path,
        public_key_path,
        postgres_url,
        redis_url,
        ethereum_rpc_url,
    );
}

extern crate env_logger;

extern crate config;
extern crate server;

use std::env;

use config::Config;
use server::server as web_server;

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config = Config::new();
    web_server::run(config);
}

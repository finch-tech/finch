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

    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST environment variable must be set.");
    let smtp_port = env::var("SMTP_PORT")
        .expect("SMTP_PORT environment variable must be set.")
        .parse::<u16>()
        .expect("Invalid smtp port.");
    let smtp_user = env::var("SMTP_USER").expect("SMTP_USER environment variable must be set.");
    let smtp_pass = env::var("SMTP_PASS").expect("SMTP_PASS environment variable must be set.");
    let registration_mail_sender = env::var("REGISTRATION_MAIL_SENDER")
        .expect("REGISTRATION_MAIL_SENDER environment variable must be set.");

    let web_client_url =
        env::var("WEB_CLIENT_URL").expect("WEB_CLIENT_URL environment variable must be set.");

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
        smtp_host,
        smtp_port,
        smtp_user,
        smtp_pass,
        registration_mail_sender,
        web_client_url,
    );
}

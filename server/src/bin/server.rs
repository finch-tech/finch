extern crate dotenv;
extern crate env_logger;
extern crate server;

extern crate currency_api_client;

use std::str::FromStr;

use currency_api_client::Api as CurrencyApi;
use server::server as web_server;
use std::env;

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    dotenv::dotenv().ok();

    // Server host & port.
    let host = env::var("HOST").expect("HOST environment variable must be set.");
    let port = env::var("PORT").expect("PORT environment variable must be set.");

    // Key pair for authentication.
    let private_key_path =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set.");
    let public_key_path =
        env::var("PUBLIC_KEY").expect("PUBLIC_KEY environment variable must be set.");

    // Postgres url.
    let postgres_url =
        env::var("POSTGRES_URL").expect("POSTGRES_URL environment variable must be set.");

    // SMTP settings.
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST environment variable must be set.");
    let smtp_port = env::var("SMTP_PORT")
        .expect("SMTP_PORT environment variable must be set.")
        .parse::<u16>()
        .expect("Invalid smtp port.");
    let smtp_user = env::var("SMTP_USER").expect("SMTP_USER environment variable must be set.");
    let smtp_pass = env::var("SMTP_PASS").expect("SMTP_PASS environment variable must be set.");

    // Ensuring currency api type.
    CurrencyApi::from_str(
        &env::var("CURRENCY_API").expect("CURRENCY_API environment variable must be set."),
    ).unwrap();

    env::var("CURRENCY_API_KEY").expect("CURRENCY_API_KEY environment variable must be set.");
    env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL environment variable must be set.");
    env::var("WEB_CLIENT_URL").expect("WEB_CLIENT_URL environment variable must be set.");
    env::var("MAIL_SENDER").expect("MAIL_SENDER environment variable must be set.");

    web_server::run(
        host,
        port,
        private_key_path,
        public_key_path,
        postgres_url,
        smtp_host,
        smtp_port,
        smtp_user,
        smtp_pass,
    );
}

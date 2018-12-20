extern crate dotenv;

extern crate currency_api_client;
extern crate rpc_client;
extern crate types;

use std::{env, fs, str::FromStr};

use dotenv::dotenv;

use currency_api_client::{Api as CurrencyApi, Client as CurrencyApiClient};
use rpc_client::{bitcoin::RpcClient as BtcRpcClient, ethereum::RpcClient as EthRpcClient};
use types::{
    bitcoin::Network as BtcNetwork, ethereum::Network as EthNetwork, PrivateKey, PublicKey,
};

#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub postgres_url: String,
    pub jwt_private: PrivateKey,
    pub jwt_public: PublicKey,
    pub smtp: SmtpConfig,
    pub currency_api_client: CurrencyApiClient,
    pub btc_rpc_client: BtcRpcClient,
    pub btc_network: BtcNetwork,
    pub eth_rpc_client: EthRpcClient,
    pub eth_network: EthNetwork,
    pub web_client_url: String,
    pub mail_sender: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        // Server host & port.
        let host = env::var("HOST").expect("HOST environment variable must be set.");
        let port = env::var("PORT").expect("PORT environment variable must be set.");

        // Postgres url.
        let postgres_url =
            env::var("POSTGRES_URL").expect("POSTGRES_URL environment variable must be set.");

        // Bitcoin settings.
        let btc_rpc_client = BtcRpcClient::new(
            env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL environment variable must be set."),
            env::var("BITCOIN_RPC_USER")
                .expect("BITCOIN_RPC_USER environment variable must be set."),
            env::var("BITCOIN_RPC_PASS")
                .expect("BITCOIN_RPC_PASS environment variable must be set."),
        );
        let btc_network_env =
            env::var("BITCOIN_NETWORK").expect("BITCOIN_NETWORK environment variable must be set.");

        let btc_network;
        if btc_network_env == "MAINNET" {
            btc_network = BtcNetwork::MainNet;
        } else if btc_network_env == "TESTNET" {
            btc_network = BtcNetwork::TestNet
        } else {
            panic!("Invalid BITCOIN_NETWORK");
        }

        // Ethereum settings.
        let eth_rpc_client = EthRpcClient::new(
            env::var("ETHEREUM_RPC_URL")
                .expect("ETHEREUM_RPC_URL environment variable must be set."),
        );

        let eth_network_env = env::var("ETHEREUM_NETWORK")
            .expect("ETHEREUM_NETWORK environment variable must be set.");

        let eth_network;
        if eth_network_env == "MAINNET" {
            eth_network = EthNetwork::Main;
        } else if eth_network_env == "ROPSTEN" {
            eth_network = EthNetwork::Ropsten;
        } else {
            panic!("Invalid ETHEREUM_NETWORK");
        }

        // SMTP settings.
        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST environment variable must be set.");
        let smtp_port = env::var("SMTP_PORT")
            .expect("SMTP_PORT environment variable must be set.")
            .parse::<u16>()
            .expect("Invalid smtp port.");
        let smtp_user = env::var("SMTP_USER").expect("SMTP_USER environment variable must be set.");
        let smtp_pass = env::var("SMTP_PASS").expect("SMTP_PASS environment variable must be set.");

        // Keys for jwt.
        let private_key_path =
            env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set.");
        let jwt_private = fs::read(private_key_path).expect("Failed to open the private key file.");
        let public_key_path =
            env::var("PUBLIC_KEY").expect("PUBLIC_KEY environment variable must be set.");
        let jwt_public = fs::read(public_key_path).expect("Failed to open the public key file.");

        // Currency api client.
        let currency_api = CurrencyApi::from_str(
            &env::var("CURRENCY_API").expect("CURRENCY_API environment variable must be set."),
        )
        .unwrap();
        let currency_api_key = env::var("CURRENCY_API_KEY")
            .expect("CURRENCY_API_KEY environment variable must be set.");
        let currency_api_client = CurrencyApiClient::new(currency_api, currency_api_key);

        let web_client_url =
            env::var("WEB_CLIENT_URL").expect("WEB_CLIENT_URL environment variable must be set.");
        let mail_sender =
            env::var("MAIL_SENDER").expect("MAIL_SENDER environment variable must be set.");

        Config {
            host,
            port,
            postgres_url,
            jwt_private,
            jwt_public,
            smtp: {
                SmtpConfig {
                    host: smtp_host,
                    port: smtp_port,
                    user: smtp_user,
                    pass: smtp_pass,
                }
            },
            currency_api_client,
            btc_rpc_client,
            btc_network,
            eth_rpc_client,
            eth_network,
            web_client_url,
            mail_sender,
        }
    }
}

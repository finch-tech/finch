#[macro_use]
extern crate serde_derive;

extern crate currency_api_client;
extern crate types;

use currency_api_client::Api as CurrencyApi;
use types::{bitcoin::Network as BtcNetwork, ethereum::Network as EthNetwork};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub postgres: String,
    pub server: ServerConfig,
    pub smtp: SmtpConfig,
    pub bitcoin: Option<BtcConfig>,
    pub ethereum: Option<EthConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u64,
    pub private_key_path: String,
    pub public_key_path: String,
    pub mail_sender: String,
    pub web_client_url: String,
    pub currency_api: CurrencyApi,
    pub currency_api_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BtcConfig {
    pub network: BtcNetwork,
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_pass: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EthConfig {
    pub network: EthNetwork,
    pub rpc_url: String,
}

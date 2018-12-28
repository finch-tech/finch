use config::ServerConfig;
use core::db::postgres::PgExecutorAddr;
use currency_api_client::Client as CurrencyApiClient;
use mailer::MailerAddr;
use types::{
    bitcoin::Network as BtcNetwork, ethereum::Network as EthNetwork, PrivateKey, PublicKey,
};

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgExecutorAddr,
    pub mailer: MailerAddr,
    pub config: ServerConfig,
    pub jwt_public: PublicKey,
    pub jwt_private: PrivateKey,
    pub btc_network: Option<BtcNetwork>,
    pub eth_network: Option<EthNetwork>,
    pub currency_api_client: CurrencyApiClient,
}

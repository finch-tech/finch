use core::db::postgres::PgExecutorAddr;
use mailer::MailerAddr;
use types::{bitcoin::Network as BtcNetwork, PrivateKey, PublicKey};
use config::ServerConfig;
use currency_api_client::Client as CurrencyApiClient;

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgExecutorAddr,
    pub mailer: MailerAddr,
    pub config: ServerConfig,
    pub jwt_public: PublicKey,
    pub jwt_private: PrivateKey,
    pub btc_network: Option<BtcNetwork>,
    pub currency_api_client: CurrencyApiClient,
}

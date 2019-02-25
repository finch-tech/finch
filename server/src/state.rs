use config::{BtcConfig, EthConfig, ServerConfig};
use core::db::postgres::PgExecutorAddr;
use currency_api_client::CurrencyApiClientAddr;
use mailer::MailerAddr;
use types::{currency::Crypto, PrivateKey, PublicKey};

#[derive(Clone)]
pub struct AppState {
    pub postgres: PgExecutorAddr,
    pub mailer: MailerAddr,
    pub config: ServerConfig,
    pub jwt_public: PublicKey,
    pub jwt_private: PrivateKey,
    pub btc_config: Option<BtcConfig>,
    pub eth_config: Option<EthConfig>,
    pub currency_api_client: CurrencyApiClientAddr,
}

impl AppState {
    pub fn supports(&self, crypto: &Crypto) -> bool {
        match crypto {
            Crypto::Btc => self.btc_config.is_some(),
            Crypto::Eth => self.eth_config.is_some(),
        }
    }
}

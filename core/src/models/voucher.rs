use chrono::prelude::*;
use jwt;
use serde_json;
use uuid::Uuid;

use bigdecimal::BigDecimal;
use models::{payment::Payment, Error};
use types::{
    bitcoin::Network as BtcNetwork,
    currency::{Crypto, Fiat},
    ethereum::Network as EthNetwork,
    H256,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Voucher {
    pub uuid: Uuid,
    pub crypto: Crypto,
    pub tx_hash: H256,
    pub charge: BigDecimal,
    pub amount_paid: BigDecimal,
    pub fiat: Fiat,
    pub price: BigDecimal,
    pub store_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btc_network: Option<BtcNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_network: Option<EthNetwork>,
    pub exp: u64,
}

impl Voucher {
    pub fn from_payment(payment: Payment, exp: DateTime<Utc>) -> Self {
        Voucher {
            uuid: Uuid::new_v4(),
            crypto: payment.crypto,
            tx_hash: payment.transaction_hash.unwrap(),
            charge: payment.charge,
            amount_paid: payment.amount_paid.unwrap(),
            fiat: payment.fiat,
            price: payment.price,
            store_id: payment.store_id,
            btc_network: payment.btc_network,
            eth_network: payment.eth_network,
            exp: exp.timestamp() as u64,
        }
    }

    pub fn encode(&self, private_key: &[u8]) -> Result<String, Error> {
        let mut header = jwt::Header::default();
        header.alg = jwt::Algorithm::RS256;

        jwt::encode(&header, &self, private_key).map_err(|e| Error::from(e))
    }

    pub fn export(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

use chrono::prelude::*;
use jwt;
use serde_json::Value;
use uuid::Uuid;

use models::ethereum::Transaction;
use models::payment::Payment;
use models::Error;
use types::{H160, H256};

#[derive(Debug, Serialize, Deserialize)]
pub struct Voucher {
    pub tx_hash: H256,
    pub uuid: Uuid,
    pub value: String,
    // pub paid_by: H160,
    pub store_id: Uuid,
    pub exp: u64,
}

impl Voucher {
    pub fn new(payment: Payment, exp: DateTime<Utc>) -> Self {
        Voucher {
            tx_hash: payment.transaction_hash.unwrap(),
            uuid: Uuid::new_v4(),
            value: format!("{}", payment.price.unwrap()),
            // paid_by: payment.from,
            store_id: payment.store_id,
            // iss: String::from(""),
            // iat: Utc::now().timestamp(),
            exp: exp.timestamp() as u64,
        }
    }

    pub fn encode(&self, private_key: &[u8]) -> Result<String, Error> {
        let mut header = jwt::Header::default();
        header.alg = jwt::Algorithm::RS256;

        jwt::encode(&header, &self, private_key).map_err(|e| Error::from(e))
    }

    pub fn export(&self) -> Value {
        json!({
            "tx_hash": self.tx_hash,
            "uuid": self.uuid,
            "value": self.value,
            // "paid_by": self.paid_by,
            "store_id": self.store_id,
        })
    }
}

use models::bitcoin::Transaction;
use types::{H256, U128};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Block {
    pub hash: H256,
    pub height: Option<U128>,
    pub version: u32,
    pub merkleroot: H256,
    #[serde(rename = "tx")]
    pub transactions: Option<Vec<Transaction>>,
    pub time: u32,
    pub nonce: u32,
    pub previousblockhash: Option<H256>,
}

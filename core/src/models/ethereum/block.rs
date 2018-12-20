use models::ethereum::Transaction;
use types::{H160, H256, U128, U256};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: Option<H256>,
    #[serde(rename = "parentHash")]
    pub parent_hash: H256,
    #[serde(rename = "sha3Uncles")]
    pub uncles_hash: H256,
    #[serde(rename = "miner")]
    pub author: H160,
    #[serde(rename = "stateRoot")]
    pub state_root: H256,
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: H256,
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: H256,
    pub number: Option<u128>,
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    #[serde(rename = "gasLimit")]
    pub gas_limit: U256,
    #[serde(rename = "extraData")]
    pub extra_data: String,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub timestamp: U256,
    pub difficulty: U256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub hash: Option<H256>,
    #[serde(rename = "parentHash")]
    pub parent_hash: H256,
    #[serde(rename = "sha3Uncles")]
    pub uncles_hash: H256,
    #[serde(rename = "miner")]
    pub author: Option<H160>,
    #[serde(rename = "stateRoot")]
    pub state_root: H256,
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: H256,
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: H256,
    pub number: Option<U128>,
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    #[serde(rename = "gasLimit")]
    pub gas_limit: U256,
    #[serde(rename = "extraData")]
    pub extra_data: String,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: String,
    pub timestamp: U256,
    pub difficulty: U256,
    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: Option<U256>,
    pub uncles: Vec<H256>,
    pub transactions: Vec<Transaction>,
    pub size: Option<U256>,
}

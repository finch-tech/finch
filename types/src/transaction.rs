use H160;
use H256;
use U256;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Transaction {
    pub hash: H256,
    pub nonce: U256,
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<String>,
    pub from: H160,
    pub to: Option<H160>,
    pub value: U256,
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    pub gas: U256,
    pub input: String,
}

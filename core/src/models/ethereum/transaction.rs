use futures::Future;

use db::postgres::PgExecutorAddr;
use db::ethereum::transactions::{FindByHash, Insert};
use models::Error;
use schema::eth_transactions;
use types::{H160, H256, U256};

#[derive(Debug, Insertable, Queryable, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[table_name = "eth_transactions"]
pub struct Transaction {
    pub hash: H256,
    pub nonce: U256,
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<String>,
    #[serde(rename = "from")]
    pub from_address: H160,
    #[serde(rename = "to")]
    pub to_address: Option<H160>,
    pub value: U256,
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    pub gas: U256,
    pub input: String,
}

impl Transaction {
    pub fn insert(
        payload: Transaction,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Transaction, Error = Error> {
        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_hash(
        hash: H256,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Transaction, Error = Error> {
        (*postgres)
            .send(FindByHash(hash))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }
}

use futures::Future;

use db::{
    bitcoin::transactions::{FindByTxId, Insert},
    postgres::PgExecutorAddr,
};
use models::Error;
use types::H256;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ScriptType {
    #[serde(rename = "nonstandard")]
    NonStandard,
    #[serde(rename = "pubkey")]
    PubKey,
    #[serde(rename = "pubkeyhash")]
    PubKeyHash,
    #[serde(rename = "scripthash")]
    ScriptHash,
    #[serde(rename = "multisig")]
    Multisig,
    #[serde(rename = "nulldata")]
    NullData,
    #[serde(rename = "witness_v0_scripthash")]
    WitnessScript,
    #[serde(rename = "witness_v0_keyhash")]
    WitnessKey,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransactionInputScript {
    pub asm: String,
    pub hex: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransactionOutputScript {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,
    #[serde(rename = "type")]
    pub script_type: ScriptType,
    pub addresses: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SignedTransactionInput {
    pub txid: Option<H256>,
    pub vout: Option<u32>,
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<TransactionInputScript>,
    pub sequence: u32,
    pub txinwitness: Option<Vec<String>>,
    pub coinbase: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SignedTransactionOutput {
    pub value: f64,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script: TransactionOutputScript,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Transaction {
    pub txid: H256,
    pub hex: String,
    pub hash: H256,
    pub size: usize,
    pub vsize: usize,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<SignedTransactionInput>,
    pub vout: Vec<SignedTransactionOutput>,
    pub blockhash: Option<H256>,
    pub confirmations: Option<u32>,
    pub time: Option<u32>,
    pub blocktime: Option<u32>,
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

    pub fn find_by_txid(
        txid: H256,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Transaction, Error = Error> {
        (*postgres)
            .send(FindByTxId(txid))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }
}

use hex::FromHex;
use serde::{Deserialize, Deserializer};

use types::H256;

fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| Vec::from_hex(&string).map_err(|err| Error::custom(err.to_string())))
}

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
    #[serde(deserialize_with = "from_hex")]
    pub hex: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransactionOutputScript {
    pub asm: String,
    #[serde(deserialize_with = "from_hex")]
    pub hex: Vec<u8>,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,
    #[serde(rename = "type")]
    pub script_type: ScriptType,
    pub addresses: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SignedTransactionInput {
    pub txid: H256,
    pub vout: u32,
    #[serde(rename = "scriptSig")]
    pub script_sig: TransactionInputScript,
    pub sequence: u32,
    pub txinwitness: Option<Vec<String>>,
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
    #[serde(deserialize_with = "from_hex")]
    pub hex: Vec<u8>,
    pub txid: H256,
    pub hash: H256,
    pub size: usize,
    pub vsize: usize,
    pub version: i32,
    pub locktime: i32,
    pub vin: Vec<SignedTransactionInput>,
    pub vout: Vec<SignedTransactionOutput>,
    pub blockhash: H256,
    pub confirmations: u32,
    pub time: u32,
    pub blocktime: u32,
}

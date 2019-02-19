use std::ops::Deref;

use byteorder::{LittleEndian, WriteBytesExt};
use rust_base58::FromBase58;
use rustc_hex::FromHex;
use secp256k1::{
    key::{PublicKey, SecretKey},
    Message, Secp256k1, Signature,
};

use core::bitcoin::Transaction;
use types::{bitcoin::VarInt, H256};

#[derive(Default, Debug, Clone)]
pub struct Script(pub Vec<u8>);

impl Script {
    pub fn p2pkh(to: String) -> Self {
        let decoded = to.from_base58().unwrap();
        let mut pkh = decoded[1..21].to_vec();

        let mut script = Vec::new();
        script.push(OP_DUP);
        script.push(OP_HASH160);
        script.push(OP_PUSHBYTES_20);
        script.append(&mut pkh);
        script.push(OP_EQUALVERIFY);
        script.push(OP_CHECKSIG);
        Script(script)
    }

    pub fn script_sig(sig: Signature, pkey: PublicKey) -> Self {
        let secp = Secp256k1::new();

        let mut der_sig = sig.serialize_der(&secp);
        der_sig.push(0x01); //SIGHASH ALL

        let mut script = Vec::new();
        script.write_u8(der_sig.len() as u8).unwrap();
        script.extend_from_slice(der_sig.as_slice());
        script.write_u8(pkey.serialize().len() as u8).unwrap();
        script.extend_from_slice(&pkey.serialize());
        Script(script)
    }

    pub fn from_hex(hex: String) -> Self {
        let bytes = hex.from_hex().unwrap();
        Script(bytes)
    }
}

impl Deref for Script {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
const OP_DUP: u8 = 0x76;
const OP_HASH160: u8 = 0xa9;
const OP_EQUALVERIFY: u8 = 0x88;
const OP_CHECKSIG: u8 = 0xac;
const OP_PUSHBYTES_20: u8 = 0x14;

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub hash: H256,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub outpoint: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
    pub script_witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub value: u64,
    pub script_pubkey: Script,
}

#[derive(Debug, Clone)]
pub struct UnsignedTransaction {
    pub version: i32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

impl UnsignedTransaction {
    pub fn new(inputs: Vec<(Transaction, u32)>, outputs: Vec<(String, u64)>) -> Self {
        let mut tx = UnsignedTransaction {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        };

        for (utxo, index) in inputs {
            let hex_script = utxo.vout[index as usize].script.hex.clone();
            let previous_script_pubkey = Script::from_hex(hex_script);

            let input = Input {
                outpoint: OutPoint {
                    hash: utxo.txid,
                    index: index,
                },
                script_sig: previous_script_pubkey,
                sequence: 0xFFFFFFFF,
                script_witness: Vec::new(),
            };

            tx.inputs.push(input);
        }

        for (address, amount) in outputs {
            let output = Output {
                value: amount,
                script_pubkey: Script::p2pkh(address),
            };
            tx.outputs.push(output);
        }

        tx
    }

    pub fn sign(&mut self, skey: SecretKey, pkey: PublicKey) {
        for (idx, _) in self.inputs.clone().iter().enumerate() {
            let hash = self.signature_hash(idx);

            let secp = Secp256k1::new();
            let signature = secp.sign(&Message::from(hash.0), &skey);
            self.inputs[idx].script_sig = Script::script_sig(signature, pkey);
        }
    }

    pub fn signature_hash(&self, _: usize) -> H256 {
        let tx = self.clone();

        let mut serialized = Vec::new();
        tx.serialize(&mut serialized);
        serialized.write_u32::<LittleEndian>(1).unwrap(); // SIGHASH ALL
        H256::from_data(&serialized)
    }

    pub fn serialize(&self, stream: &mut Vec<u8>) {
        stream
            .write_u32::<LittleEndian>(self.version as u32)
            .unwrap();

        VarInt::from(self.inputs.len()).serialize(stream);

        for mut input in self.inputs.clone() {
            input.outpoint.hash.reverse();
            stream.extend_from_slice(&input.outpoint.hash);
            stream
                .write_u32::<LittleEndian>(input.outpoint.index)
                .unwrap();

            let script_length = VarInt::from(input.script_sig.len());
            script_length.serialize(stream);

            stream.extend_from_slice(&input.script_sig);
            stream.write_u32::<LittleEndian>(input.sequence).unwrap();
        }

        VarInt::from(self.outputs.len()).serialize(stream);

        for output in self.outputs.clone() {
            stream.write_u64::<LittleEndian>(output.value).unwrap();
            let script_length = VarInt::from(output.script_pubkey.len());
            script_length.serialize(stream);
            stream.extend_from_slice(&output.script_pubkey);
        }

        stream.write_u32::<LittleEndian>(self.lock_time).unwrap();
    }

    pub fn into_raw_transaction(&self) -> Vec<u8> {
        let mut s = Vec::new();
        self.serialize(&mut s);
        s
    }
}

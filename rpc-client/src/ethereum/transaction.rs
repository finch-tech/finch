use rlp::RlpStream;
use rustc_hex::ToHex;
use secp256k1::{Message, Secp256k1, key::SecretKey};
use tiny_keccak::keccak256;

use errors::Error;
use ethereum::Signature;

use types::{H160, H256, U128, U256};

#[derive(Debug)]
pub struct UnsignedTransaction {
    pub nonce: U128,
    pub gas_price: U256,
    pub gas: U256,
    pub to: H160,
    pub value: U256,
    pub data: Vec<u8>,
}

impl UnsignedTransaction {
    pub fn sign(self, secret_key: SecretKey, chain_id: u64) -> Result<SignedTransaction, Error> {
        let mut stream = RlpStream::new();

        stream.begin_list(9);
        stream.append(&self.nonce);
        stream.append(&self.gas_price);
        stream.append(&self.gas);
        stream.append(&self.to);
        stream.append(&self.value);
        stream.append(&self.data);
        stream.append(&chain_id);
        stream.append(&0u8);
        stream.append(&0u8);

        let hash = H256::from_hash(keccak256(stream.as_raw()));

        let secp = Secp256k1::new();
        let s = secp.sign_recoverable(&Message::from_slice(&hash[..])?, &secret_key);
        let (rec_id, data) = s.serialize_compact(&secp);
        let mut data_arr = [0; 65];

        data_arr[0..64].copy_from_slice(&data[0..64]);
        data_arr[64] = rec_id.to_i32() as u8;
        let signature = Signature::new(data_arr);

        Ok(SignedTransaction {
            transaction: self,
            v: signature.v() as u64 + (35 + chain_id * 2),
            r: signature.r().into(),
            s: signature.s().into(),
        })
    }
}

#[derive(Debug)]
pub struct SignedTransaction {
    transaction: UnsignedTransaction,
    v: u64,
    r: U256,
    s: U256,
}

impl SignedTransaction {
    pub fn rlp_encode(&self) -> String {
        let mut s = RlpStream::new();
        s.begin_list(9);
        s.append(&self.transaction.nonce);
        s.append(&self.transaction.gas_price);
        s.append(&self.transaction.gas);
        s.append(&self.transaction.to);
        s.append(&self.transaction.value);
        s.append(&self.transaction.data);
        s.append(&self.v);
        s.append(&self.r);
        s.append(&self.s);

        s.drain().into_vec().to_hex()
    }
}

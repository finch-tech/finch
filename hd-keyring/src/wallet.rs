use rust_base58::ToBase58;
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::Secp256k1;
use tiny_keccak::keccak256;

use errors::Error;
use types::{H160, H256};

#[derive(Debug)]
pub struct Wallet {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Wallet {
    pub fn from_secret_key(secret_key: SecretKey) -> Result<Self, Error> {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key)?;

        Ok(Wallet {
            secret_key,
            public_key,
        })
    }

    pub fn get_eth_address(&self) -> String {
        let key_hash = keccak256(&self.public_key.serialize_uncompressed()[1..]); // Ignoring prefix 0x04.

        let mut address = String::from("0x");
        address.push_str(
            &key_hash[key_hash.len() - 20..]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>(),
        );

        address
    }

    pub fn get_btc_address(&self) -> String {
        // h160 on public key.
        let h160 = H160::from_data(&self.public_key.serialize()[..]);

        // Add version prefix.
        let mut prefixed = [0; 21];
        // TODO: Network check.
        // prefixed[0] = match self.network {
        //     Network::Bitcoin => 0,
        //     Network::Testnet | Network::Regtest => 111,
        // };
        prefixed[0] = 0;
        prefixed[1..].copy_from_slice(&h160[..]);

        // h256 on prefixed h160.
        let h256 = H256::from_data(&prefixed);

        // 25 byte binary Bitcoin Address.
        let mut address = [0; 25];
        address[0..21].copy_from_slice(&prefixed);
        address[21..].copy_from_slice(&h256[0..4]);

        // Base58 string of the address.
        address.to_base58()
    }
}

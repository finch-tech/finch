use std::str::FromStr;

use rust_base58::ToBase58;
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::Secp256k1;
use tiny_keccak::keccak256;

use errors::Error;
use types::{bitcoin::Network as BtcNetwork, Currency, H160, H256};

#[derive(Debug)]
pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub btc_network: BtcNetwork,
}

impl Wallet {
    pub fn from_secret_key(secret_key: SecretKey, btc_network: BtcNetwork) -> Result<Self, Error> {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Wallet {
            secret_key,
            public_key,
            btc_network: btc_network,
        })
    }

    pub fn get_address(&self, currency: &Currency) -> String {
        match currency {
            Currency::Btc => self.get_btc_address(),
            Currency::Eth => format!("0x{}", self.get_eth_address()),
            _ => panic!("Invalid currency for wallet"),
        }
    }

    pub fn get_eth_address(&self) -> H160 {
        let key_hash = keccak256(&self.public_key.serialize_uncompressed()[1..]); // Ignoring prefix 0x04.

        let mut address = String::new();
        address.push_str(
            &key_hash[key_hash.len() - 20..]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>(),
        );

        H160::from_str(&address).unwrap()
    }

    pub fn get_btc_address(&self) -> String {
        // h160 on public key.
        let h160 = H160::from_data(&self.public_key.serialize()[..]);

        // Add version prefix.
        let mut prefixed = [0; 21];

        prefixed[0] = match self.btc_network {
            BtcNetwork::MainNet => 0,
            BtcNetwork::TestNet => 111,
        };

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

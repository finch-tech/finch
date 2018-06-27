use std::default::Default;
use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;
use std::string::ToString;

use bip39::Seed;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use hmac::{Hmac, Mac};
use regex::Regex;
use rust_base58::{FromBase58, ToBase58};
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::Secp256k1;
use sha2::Sha512;

use errors::Error;
use types::{H160, H256};

const MASTER_SECRET: &'static [u8] = b"Bitcoin seed";
const HARDENED_OFFSET: u32 = 0x80000000;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Network {
    BITCOIN_MAINNET,
    BITCOIN_TESTNET,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Fingerprint([u8; 4]);

impl Default for Fingerprint {
    fn default() -> Fingerprint {
        Fingerprint([0, 0, 0, 0])
    }
}

impl Deref for Fingerprint {
    type Target = [u8; 4];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for Fingerprint {
    fn from(fingerprint: &'a [u8]) -> Self {
        let mut slice = [0; 4];
        slice.copy_from_slice(&fingerprint[..]);
        Fingerprint(slice)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChainCode([u8; 32]);

impl<'a> From<&'a [u8]> for ChainCode {
    fn from(chain_code: &'a [u8]) -> Self {
        let mut slice = [0; 32];
        slice.copy_from_slice(&chain_code[..]);
        ChainCode(slice)
    }
}

impl Deref for ChainCode {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Index {
    Hard(u32),
    Soft(u32),
}

impl Index {
    pub fn into_inner(&self) -> u32 {
        match *self {
            Index::Hard(n) => n,
            Index::Soft(n) => n,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DerivationPath(Vec<Index>);

impl Deref for DerivationPath {
    type Target = Vec<Index>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for DerivationPath {
    type Err = Error;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let entries: Vec<&str> = path.split('/').collect();

        let mut index_list = Vec::new();

        for (i, c) in entries.iter().enumerate() {
            if i == 0 {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"^[mM]{1}$").expect("Invalid regex pattern");
                }
                if RE.is_match(c) == false {
                    return Err(Error::InvalidDerivationPath);
                }
                continue;
            }

            let mut index;
            let mut di;

            if (c.len() > 1) && (c.ends_with('\'')) {
                let (c, _) = c.split_at(c.len() - 1);
                index = c.parse::<u32>().unwrap();
                index += HARDENED_OFFSET;
                di = Index::Hard(index);
            } else {
                index = c.parse::<u32>().unwrap();
                di = Index::Soft(index);
            }

            index_list.push(di);
        }

        Ok(DerivationPath(index_list))
    }
}

impl ToString for DerivationPath {
    fn to_string(&self) -> String {
        let index_list = self.0.clone();
        let mut path = String::from("m");
        for (_, index) in index_list.iter().enumerate() {
            match index {
                Index::Hard(n) => {
                    path.push('/');
                    path.push_str(&(n - HARDENED_OFFSET).to_string());
                    path.push('\'');
                }
                Index::Soft(n) => path.push_str(&n.to_string()),
            }
        }

        path
    }
}

#[derive(Debug)]
pub struct XKeyPair {
    xprv: Xprv,
    xpub: Xpub,
}

impl XKeyPair {
    pub fn from_seed(seed: Seed) -> Result<Self, Error> {
        let xprv = Xprv::from_master_seed(seed)?;
        let xpub = Xpub::from_private(&xprv)?;

        Ok(XKeyPair { xprv, xpub })
    }

    pub fn from_path(&self, path: &DerivationPath) -> Result<Self, Error> {
        let xprv = self.xprv.derive(path)?;

        Ok(XKeyPair {
            xprv,
            xpub: Xpub::from_private(&xprv)?,
        })
    }

    pub fn derive(&self, index: &Index) -> Result<XKeyPair, Error> {
        let xprv = self.xprv.ckd_priv(index)?;
        let xpub = Xpub::from_private(&xprv)?;

        Ok(XKeyPair { xprv, xpub })
    }

    pub fn xprv(&self) -> &Xprv {
        &self.xprv
    }

    pub fn xpub(&self) -> &Xpub {
        &self.xpub
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Xprv {
    secret_key: SecretKey,
    chain_code: ChainCode,
    network: Network,
    depth: u32,
    index: Index,
    parent_fingerprint: Fingerprint,
}

impl Xprv {
    pub fn from_master_seed(seed: Seed) -> Result<Self, Error> {
        let mut mac = Hmac::<Sha512>::new_varkey(MASTER_SECRET).unwrap();

        mac.input(seed.as_bytes());
        let i = mac.result().code();

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&secp, &i[..32])?;

        Ok(Xprv {
            secret_key,
            chain_code: ChainCode::from(&i[32..]),
            depth: 0,
            network: Network::BITCOIN_MAINNET,
            index: Index::Soft(0),
            parent_fingerprint: Default::default(),
        })
    }

    pub fn derive(&self, path: &DerivationPath) -> Result<Self, Error> {
        let mut xprv = *self;

        for (_, index) in path.iter().enumerate() {
            xprv = xprv.ckd_priv(index)?;
        }

        Ok(xprv)
    }

    pub fn ckd_priv(&self, index: &Index) -> Result<Self, Error> {
        let mut mac = Hmac::<Sha512>::new_varkey(&self.chain_code).unwrap();
        let secp = Secp256k1::new();

        match *index {
            Index::Hard(_) => {
                mac.input(&[0u8]);
                mac.input(&self.secret_key[..]);
            }
            Index::Soft(_) => {
                let public_key = PublicKey::from_secret_key(&secp, &self.secret_key)?;
                mac.input(&public_key.serialize()[..]);
            }
        };

        let mut index_bytes = [0; 4];
        BigEndian::write_u32(&mut index_bytes, index.into_inner());
        mac.input(&index_bytes);

        let result = mac.result().code();

        let mut secret_key = SecretKey::from_slice(&secp, &result[..32])?;
        secret_key.add_assign(&secp, &self.secret_key)?;

        Ok(Xprv {
            depth: self.depth + 1,
            parent_fingerprint: self.fingerprint(&secp)?,
            secret_key,
            network: self.network,
            index: *index,
            chain_code: ChainCode::from(&result[32..]),
        })
    }

    pub fn identifier(&self, secp: &Secp256k1) -> Result<H160, Error> {
        let public_key = PublicKey::from_secret_key(secp, &self.secret_key)?;
        Ok(H160::from_data(&public_key.serialize()[..]))
    }

    pub fn fingerprint(&self, secp: &Secp256k1) -> Result<Fingerprint, Error> {
        let h160 = self.identifier(secp)?;
        Ok(Fingerprint::from(&h160[0..4])) // Using first 4 bytes
    }

    pub fn as_raw(&self) -> &SecretKey {
        &self.secret_key
    }
}

impl ToString for Xprv {
    fn to_string(&self) -> String {
        let mut data = [0; 78];
        data[0..4].copy_from_slice(
            &match self.network {
                Network::BITCOIN_MAINNET => [0x04u8, 0x88, 0xAD, 0xE4],
                Network::BITCOIN_TESTNET => [0x04u8, 0x35, 0x83, 0x94],
            }[..],
        );
        data[4] = self.depth as u8;
        data[5..9].copy_from_slice(&self.parent_fingerprint[..]);
        BigEndian::write_u32(&mut data[9..13], self.index.into_inner());
        data[13..45].copy_from_slice(&self.chain_code[..]);
        data[45] = 0;
        data[46..78].copy_from_slice(&self.secret_key[..]);

        let checksum = H256::from_data(&data);

        let mut concatenated = vec![];
        concatenated.extend_from_slice(&data);
        concatenated.extend_from_slice(&checksum[0..4]);

        concatenated.to_base58()
    }
}

impl FromStr for Xprv {
    type Err = Error;

    fn from_str(input: &str) -> Result<Xprv, Self::Err> {
        let s = Secp256k1::with_caps(secp256k1::ContextFlag::None);

        let bytes = input.from_base58().map_err(|_| Error::InvalidBase58Byte)?;

        let expected = LittleEndian::read_u32(&H256::from_data(&bytes[..bytes.len() - 4])[0..4]);
        let actual = LittleEndian::read_u32(&bytes[bytes.len() - 4..]);

        if expected != actual {
            return Err(Error::BadChecksum);
        }

        let data = &bytes[..bytes.len() - 4][..];

        if data.len() != 78 {
            return Err(Error::InvalidKeyLength);
        }

        let n = Cursor::new(&data[9..13]).read_u32::<BigEndian>()?;
        let index = if n < HARDENED_OFFSET {
            Index::Soft(n)
        } else {
            Index::Hard(n)
        };

        Ok(Xprv {
            network: if &data[0..4] == [0x04u8, 0x88, 0xAD, 0xE4] {
                Network::BITCOIN_MAINNET
            } else if &data[0..4] == [0x04u8, 0x35, 0x83, 0x94] {
                Network::BITCOIN_TESTNET
            } else {
                return Err(Error::InvalidNetwork);
            },
            depth: data[4] as u32,
            parent_fingerprint: Fingerprint::from(&data[5..9]),
            index,
            chain_code: ChainCode::from(&data[13..45]),
            secret_key: SecretKey::from_slice(&s, &data[46..78])?,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Xpub {
    public_key: PublicKey,
    chain_code: ChainCode,
    network: Network,
    depth: u32,
    index: Index,
    parent_fingerprint: Fingerprint,
}

impl Xpub {
    pub fn from_private(xprv: &Xprv) -> Result<Xpub, Error> {
        let secp = Secp256k1::new();

        Ok(Xpub {
            network: xprv.network,
            depth: xprv.depth,
            index: xprv.index,
            parent_fingerprint: xprv.parent_fingerprint,
            public_key: PublicKey::from_secret_key(&secp, &xprv.secret_key)?,
            chain_code: xprv.chain_code,
        })
    }

    pub fn ckd_pub(&self, index: &Index) -> Result<Self, Error> {
        let mut mac = Hmac::<Sha512>::new_varkey(&self.chain_code).unwrap();
        let secp = Secp256k1::new();

        match *index {
            Index::Hard(_) => return Err(Error::InvalidDerivation),
            Index::Soft(_) => {
                mac.input(&self.public_key.serialize()[..]);
            }
        };

        let mut index_bytes = [0; 4];
        BigEndian::write_u32(&mut index_bytes, index.into_inner());
        mac.input(&index_bytes);

        let result = mac.result().code();

        let secret_key = SecretKey::from_slice(&secp, &result[..32])?;
        let mut public_key = self.public_key.clone();
        public_key.add_exp_assign(&secp, &secret_key)?;

        Ok(Xpub {
            depth: self.depth + 1,
            index: *index,
            network: self.network,
            parent_fingerprint: self.fingerprint(),
            public_key,
            chain_code: ChainCode::from(&result[32..]),
        })
    }

    pub fn identifier(&self) -> H160 {
        H160::from_data(&self.public_key.serialize()[..])
    }

    pub fn fingerprint(&self) -> Fingerprint {
        let h160 = self.identifier();
        Fingerprint::from(&h160[0..4]) // Using first 4 bytes
    }

    pub fn as_raw(&self) -> &PublicKey {
        &self.public_key
    }
}

impl ToString for Xpub {
    fn to_string(&self) -> String {
        let mut data = [0; 78];
        data[0..4].copy_from_slice(
            &match self.network {
                Network::BITCOIN_MAINNET => [0x04u8, 0x88, 0xB2, 0x1E],
                Network::BITCOIN_TESTNET => [0x04u8, 0x35, 0x87, 0xCF],
            }[..],
        );
        data[4] = self.depth as u8;
        data[5..9].copy_from_slice(&self.parent_fingerprint[..]);
        BigEndian::write_u32(&mut data[9..13], self.index.into_inner());
        data[13..45].copy_from_slice(&self.chain_code[..]);
        data[45..78].copy_from_slice(&self.public_key.serialize()[..]);

        let checksum = H256::from_data(&data);

        let mut concatenated = vec![];
        concatenated.extend_from_slice(&data);
        concatenated.extend_from_slice(&checksum[0..4]);

        concatenated.to_base58()
    }
}

impl FromStr for Xpub {
    type Err = Error;

    fn from_str(input: &str) -> Result<Xpub, Self::Err> {
        let s = Secp256k1::with_caps(secp256k1::ContextFlag::None);

        let bytes = input.from_base58().map_err(|_| Error::InvalidBase58Byte)?;

        let expected = LittleEndian::read_u32(&H256::from_data(&bytes[..bytes.len() - 4])[0..4]);
        let actual = LittleEndian::read_u32(&bytes[bytes.len() - 4..]);

        if expected != actual {
            return Err(Error::BadChecksum);
        }

        let data = &bytes[..bytes.len() - 4][..];

        if data.len() != 78 {
            return Err(Error::InvalidKeyLength);
        }

        let n = Cursor::new(&data[9..13]).read_u32::<BigEndian>()?;
        let index = if n < HARDENED_OFFSET {
            Index::Soft(n)
        } else {
            Index::Hard(n)
        };

        Ok(Xpub {
            network: if &data[0..4] == [0x04u8, 0x88, 0xB2, 0x1E] {
                Network::BITCOIN_MAINNET
            } else if &data[0..4] == [0x04u8, 0x35, 0x87, 0xCF] {
                Network::BITCOIN_TESTNET
            } else {
                return Err(Error::InvalidNetwork);
            },
            depth: data[4] as u32,
            parent_fingerprint: Fingerprint::from(&data[5..9]),
            index,
            chain_code: ChainCode::from(&data[13..45]),
            public_key: PublicKey::from_slice(&s, &data[45..78])?,
        })
    }
}

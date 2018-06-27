#![crate_name = "hd_keyring"]
#![feature(rustc_private, extern_prelude)]

extern crate bip39;
extern crate byteorder;
extern crate digest;
extern crate hmac;
extern crate regex;
extern crate ripemd160;
extern crate secp256k1;
extern crate sha2;
extern crate tiny_keccak;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate rust_base58;

extern crate types;

mod bip32;
mod errors;
mod keyring;
mod wallet;

pub use bip32::{DerivationPath, Index, XKeyPair, Xprv, Xpub};
pub use errors::Error;
pub use keyring::HdKeyring;
pub use wallet::Wallet;

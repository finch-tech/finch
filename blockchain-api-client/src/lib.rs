#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate byteorder;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate rlp;
extern crate rust_base58;
extern crate rustc_hex;
extern crate secp256k1;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tiny_keccak;

extern crate core;
extern crate types;

pub mod bitcoin;
pub mod errors;
pub mod ethereum;

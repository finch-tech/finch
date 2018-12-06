#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate byteorder;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate tokio;

extern crate config;
extern crate core;
extern crate hd_keyring;
extern crate rpc_client;
extern crate types;

pub mod bitcoin;
pub mod errors;
pub mod ethereum;

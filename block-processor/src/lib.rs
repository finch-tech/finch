#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate bigdecimal;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_timer;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
extern crate tokio;

extern crate core;
extern crate rpc_client;
extern crate types;

pub mod bitcoin;
pub mod ethereum;

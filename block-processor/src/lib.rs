#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate bigdecimal;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_timer;
extern crate serde;
extern crate serde_json;
extern crate tokio;

extern crate core;
extern crate eth_rpc_client;
extern crate types;

mod errors;
mod poller;
mod processor;

pub mod service;

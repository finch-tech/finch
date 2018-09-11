#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate bigdecimal;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;

extern crate core;
extern crate ethereum_client;
extern crate types;

mod consumer;
mod subscriber;

pub mod service;

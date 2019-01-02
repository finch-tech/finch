#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bigdecimal;
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate hex;
extern crate jsonwebtoken as jwt;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis as _redis;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

extern crate currency_api_client;
extern crate types;

mod schema;

pub mod db;
mod models;

pub use models::{
    bitcoin, client_token, ethereum, payment, payout, store, user, voucher, Error as ModelError,
};

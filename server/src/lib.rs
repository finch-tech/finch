#![feature(pattern_parentheses, rustc_private)]

#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate chrono;
extern crate data_encoding;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate jsonwebtoken as jwt;
extern crate num_cpus;
extern crate openssl;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate ring;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate secp256k1;
extern crate uuid;

extern crate hd_keyring;
extern crate types;

mod auth;
mod controllers;
mod db;
mod models;
mod schema;
mod services;

pub mod server;

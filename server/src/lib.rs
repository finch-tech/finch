extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bigdecimal;
extern crate chrono;
extern crate data_encoding;
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate jsonwebtoken as jwt;
extern crate lettre;
extern crate lettre_email;
extern crate native_tls;
extern crate num_cpus;
extern crate openssl;
extern crate ring;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate secp256k1;
extern crate uuid;

extern crate core;
extern crate currency_api_client;
extern crate ethereum_client;
extern crate hd_keyring;
extern crate types;

mod auth;
mod controllers;
mod mailer;
mod services;

pub mod server;

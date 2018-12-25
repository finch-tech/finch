#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix_web;
extern crate bigdecimal;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

extern crate types;

mod api;
mod client;
mod errors;

pub use self::api::Api;
pub use self::client::Client;
pub use self::errors::Error;

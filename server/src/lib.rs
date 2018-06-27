#![feature(pattern_parentheses, rustc_private)]

#[macro_use]
extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate num_cpus;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
#[macro_use]
extern crate serde_json;
extern crate types;

mod controllers;
mod db;

pub mod server;

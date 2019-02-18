#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate bigdecimal;
extern crate chrono;
extern crate data_encoding;
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate env_logger;
extern crate futures;
extern crate jsonwebtoken as jwt;
extern crate lettre;
extern crate lettre_email;
extern crate log;
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

extern crate config;
extern crate core;
extern crate currency_api_client;
extern crate hd_keyring;
extern crate rpc_client;
extern crate types;

mod auth;
mod controllers;
mod mailer;
mod services;
mod state;

use std::fs;

use actix::prelude::*;
use actix_web::{http, middleware, server, App};

use config::Config;
use core::db::postgres;
use currency_api_client::Client as CurrencyApiClient;
use mailer::Mailer;

pub fn run(postgres: postgres::PgExecutorAddr, config: Config) {
    let smtp_config = config.smtp.clone();
    let mailer = SyncArbiter::start(num_cpus::get() * 1, move || {
        Mailer(mailer::init_mailer(
            smtp_config.host.clone(),
            smtp_config.port.clone(),
            smtp_config.user.clone(),
            smtp_config.pass.clone(),
        ))
    });

    let currency_api = config.server.currency_api.clone();
    let currency_api_key = config.server.currency_api_key.clone();
    let currency_api_client =
        Arbiter::start(move |_| CurrencyApiClient::new(&currency_api, &currency_api_key));

    let host = config.server.host.clone();
    let port = config.server.port.clone();

    server::new(move || {
        App::with_state(state::AppState {
            postgres: postgres.clone(),
            mailer: mailer.clone(),
            config: config.server.clone(),
            jwt_public: fs::read(config.server.public_key_path.clone())
                .expect("failed to open the public key file"),
            jwt_private: fs::read(config.server.private_key_path.clone())
                .expect("failed to open the private key file"),
            btc_network: {
                if let Some(btc_config) = config.bitcoin.clone() {
                    Some(btc_config.network)
                } else {
                    None
                }
            },
            eth_network: {
                if let Some(eth_config) = config.ethereum.clone() {
                    Some(eth_config.network)
                } else {
                    None
                }
            },
            currency_api_client: currency_api_client.clone(),
        })
        .middleware(middleware::Logger::default())
        .configure(|app| {
            middleware::cors::Cors::for_app(app)
                .max_age(3600)
                .resource("/", |r| {
                    r.method(http::Method::GET).with(controllers::root::index);
                })
                .resource("/registration", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::registration);
                })
                .resource("/activation", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::activation);
                })
                .resource("/login", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::authentication);
                })
                .resource("/reset_password", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::reset_password);
                })
                .resource("/change_password", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::change_password);
                })
                .resource("/profile", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::auth::profile);
                })
                .resource("/users/{id}", |r| {
                    r.method(http::Method::DELETE)
                        .with_async(controllers::auth::delete);
                })
                .resource("/client_tokens", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::client_tokens::list);
                    r.method(http::Method::POST)
                        .with_async(controllers::client_tokens::create);
                })
                .resource("/client_tokens/{id}", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::client_tokens::get);
                    r.method(http::Method::DELETE)
                        .with_async(controllers::client_tokens::delete);
                })
                .resource("/stores", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::stores::list);
                    r.method(http::Method::POST)
                        .with_async(controllers::stores::create);
                })
                .resource("/stores/{id}", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::stores::get);
                    r.method(http::Method::PATCH)
                        .with_async(controllers::stores::patch);
                    r.method(http::Method::DELETE)
                        .with_async(controllers::stores::delete);
                })
                .resource("/payments", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::payments::create);
                })
                .resource("/payments/{id}/status", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::payments::get_status)
                })
                .resource("/vouchers", |r| {
                    r.method(http::Method::POST)
                        .with_async(controllers::vouchers::create);
                })
                .register()
        })
    })
    .bind(format!("{}:{}", host, port))
    .expect(&format!("can not bind {}:{}", host, port))
    .start();
}

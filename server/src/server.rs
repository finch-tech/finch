use std::fs;

use actix::prelude::*;
use actix_web::{http, middleware, server, App};
use num_cpus;

use controllers;
use core::db::{postgres, redis};
use mailer::{self, Mailer, MailerAddr};
use types::{PrivateKey, PublicKey};

#[derive(Clone)]
pub struct AppState {
    pub postgres: postgres::PgExecutorAddr,
    pub redis: redis::RedisExecutorAddr,
    pub mailer: MailerAddr,
    pub jwt_private: PrivateKey,
    pub jwt_public: PublicKey,
    pub ethereum_rpc_url: String,
    pub web_client_url: String,
    pub registration_mail_sender: String,
}

pub fn run(
    host: String,
    port: String,
    private_key_path: String,
    public_key_path: String,
    postgres_url: String,
    redis_url: String,
    ethereum_rpc_url: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    registration_mail_sender: String,
    web_client_url: String,
) {
    System::run(move || {
        let jwt_private = fs::read(private_key_path).expect("Failed to open the private key file.");
        let jwt_public = fs::read(public_key_path).expect("Failed to open the public key file.");

        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(num_cpus::get() * 4, move || {
            postgres::PgExecutor(pg_pool.clone())
        });

        let redis_addr = SyncArbiter::start(num_cpus::get() * 1, move || {
            redis::RedisExecutor(redis::init_pool(&redis_url))
        });

        let mailer_addr = SyncArbiter::start(num_cpus::get() * 1, move || {
            Mailer(mailer::init_mailer(
                smtp_host.clone(),
                smtp_port.clone(),
                smtp_user.clone(),
                smtp_pass.clone(),
            ))
        });

        server::new(move || {
            App::with_state(AppState {
                postgres: pg_addr.clone(),
                redis: redis_addr.clone(),
                mailer: mailer_addr.clone(),
                jwt_private: jwt_private.clone(),
                jwt_public: jwt_public.clone(),
                ethereum_rpc_url: ethereum_rpc_url.clone(),
                registration_mail_sender: registration_mail_sender.clone(),
                web_client_url: web_client_url.clone(),
            }).middleware(middleware::Logger::default())
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
                .resource("/profile", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::auth::profile);
                })
                .resource("/client_tokens", |r| {
                    r.method(http::Method::GET)
                        .with_async(controllers::client_tokens::list);
                    r.method(http::Method::POST)
                        .with_async(controllers::client_tokens::create);
                })
                .resource("/client_tokens/{id}", |r| {
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
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::payments::create);
                })
                .resource("/payments/{id}/status", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET)
                        .with_async(controllers::payments::get_status)
                })
                .resource("/vouchers", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::vouchers::create);
                })
        }).bind(format!("{}:{}", host, port))
            .expect(&format!("Can not bind {}:{}", host, port))
            .start();
    });
}

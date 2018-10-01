use std::fs;

use actix::prelude::*;
use actix_web::{http, middleware, server, App};
use num_cpus;

use controllers;
use core::db::postgres;
use mailer::{self, Mailer, MailerAddr};
use types::{PrivateKey, PublicKey};

#[derive(Clone)]
pub struct AppState {
    pub postgres: postgres::PgExecutorAddr,
    pub mailer: MailerAddr,
    pub jwt_private: PrivateKey,
    pub jwt_public: PublicKey,
}

pub fn run(
    host: String,
    port: String,
    private_key_path: String,
    public_key_path: String,
    postgres_url: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
) {
    System::run(move || {
        let jwt_private = fs::read(private_key_path).expect("Failed to open the private key file.");
        let jwt_public = fs::read(public_key_path).expect("Failed to open the public key file.");

        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(num_cpus::get() * 4, move || {
            postgres::PgExecutor(pg_pool.clone())
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
                mailer: mailer_addr.clone(),
                jwt_private: jwt_private.clone(),
                jwt_public: jwt_public.clone(),
            }).middleware(middleware::Logger::default())
                .resource("/", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET).with(controllers::root::index);
                })
                .resource("/registration", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::registration);
                })
                .resource("/activation", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::activation);
                })
                .resource("/login", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::authentication);
                })
                .resource("/reset_password", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::reset_password);
                })
                .resource("/change_password", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::POST)
                        .with_async(controllers::auth::change_password);
                })
                .resource("/profile", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET)
                        .with_async(controllers::auth::profile);
                })
                .resource("/client_tokens", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET)
                        .with_async(controllers::client_tokens::list);
                    r.method(http::Method::POST)
                        .with_async(controllers::client_tokens::create);
                })
                .resource("/client_tokens/{id}", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET)
                        .with_async(controllers::client_tokens::get);
                    r.method(http::Method::DELETE)
                        .with_async(controllers::client_tokens::delete);
                })
                .resource("/stores", |r| {
                    middleware::cors::Cors::build().finish().register(r);
                    r.method(http::Method::GET)
                        .with_async(controllers::stores::list);
                    r.method(http::Method::POST)
                        .with_async(controllers::stores::create);
                })
                .resource("/stores/{id}", |r| {
                    middleware::cors::Cors::build().finish().register(r);
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

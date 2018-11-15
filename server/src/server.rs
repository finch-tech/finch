use actix::prelude::*;
use actix_web::{http, middleware, server, App};
use num_cpus;

use config::Config;
use controllers;
use core::db::postgres;
use mailer::{self, Mailer, MailerAddr};

#[derive(Clone)]
pub struct AppState {
    pub postgres: postgres::PgExecutorAddr,
    pub mailer: MailerAddr,
    pub config: Config,
}

pub fn run(config: Config) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&config.postgres_url);
        let pg_addr = SyncArbiter::start(num_cpus::get() * 4, move || {
            postgres::PgExecutor(pg_pool.clone())
        });

        let smtp_config = config.smtp.clone();
        let mailer_addr = SyncArbiter::start(num_cpus::get() * 1, move || {
            Mailer(mailer::init_mailer(
                smtp_config.host.clone(),
                smtp_config.port.clone(),
                smtp_config.user.clone(),
                smtp_config.pass.clone(),
            ))
        });

        let host = config.host.clone();
        let port = config.port.clone();

        server::new(move || {
            App::with_state(AppState {
                postgres: pg_addr.clone(),
                mailer: mailer_addr.clone(),
                config: config.clone(),
            })
            .middleware(middleware::Logger::default())
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
            .resource("/users/{id}", |r| {
                middleware::cors::Cors::build().finish().register(r);
                r.method(http::Method::DELETE)
                    .with_async(controllers::auth::delete);
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
        })
        .bind(format!("{}:{}", host, port))
        .expect(&format!("Can not bind {}:{}", host, port))
        .start();
    });
}

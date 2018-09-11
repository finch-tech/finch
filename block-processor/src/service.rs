use actix::prelude::*;
use actix_web::actix::spawn;
use actix_web::ws::Client;
use futures::Future;

use consumer::Consumer;
use core::db::postgres;
use subscriber::Subscriber;

pub fn run(postgres_url: String, ethereum_ws_url: String, ethereum_rpc_url: String) {
    System::run(move || {
        let pg_pool = postgres::init_pool(&postgres_url);
        let pg_addr = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));
        let consumer_address = Arbiter::start(move |_| Consumer {
            postgres: pg_addr.clone(),
            ethereum_rpc_url,
        });

        spawn(
            Client::new(&ethereum_ws_url)
                .max_frame_size(262_144)
                .connect()
                .map_err(|e| {
                    println!("{:?}", e);
                    ()
                })
                .map(move |(reader, writer)| {
                    let _addr: Addr<_> = Subscriber::create(move |ctx| {
                        Subscriber::add_stream(reader, ctx);
                        Subscriber::new(writer, consumer_address)
                    });
                }),
        );
    });
}

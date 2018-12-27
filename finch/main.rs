extern crate actix;
#[macro_use]
extern crate clap;
extern crate env_logger;

extern crate rpc_client;
extern crate toml;

extern crate block_processor;
extern crate config;
extern crate core;
extern crate payouter;
extern crate server;
extern crate types;

use actix::prelude::*;
use clap::App;
use std::{env, fs::File, io::prelude::*};

use config::Config;
use core::db::postgres;
use rpc_client::{bitcoin::RpcClient as BtcRpcClient, ethereum::RpcClient as EthRpcClient};
use types::Currency;

fn main() {
    env::set_var(
        "RUST_LOG",
        "info,error,debug,actix_web=info,tokio_reactor=info",
    );
    env_logger::init();

    System::run(move || {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();

        let mut settings = String::new();

        File::open(
            matches
                .value_of("settings")
                .unwrap_or(format!("{}/.finch.toml", env!("HOME")).as_str()),
        )
        .and_then(|mut f| f.read_to_string(&mut settings))
        .unwrap();

        let config: Config = toml::from_str(&settings).unwrap();

        let currencies = values_t!(matches, "currencies", Currency).unwrap();

        let postgres_url = config.postgres.clone();
        let pg_pool = postgres::init_pool(&postgres_url);
        let postgres = SyncArbiter::start(4, move || postgres::PgExecutor(pg_pool.clone()));

        let skip_missed_blocks = matches.is_present("skip_missed_blocks");

        for c in currencies {
            match c {
                Currency::Btc => {
                    use block_processor::bitcoin::service as block_processor;
                    use payouter::bitcoin::service as payouter;

                    let btc_config = config.bitcoin.clone().expect("No bitcoin configuration.");

                    let btc_conf = btc_config.clone();
                    let rpc_client = BtcRpcClient::new(
                        &btc_conf.rpc_url,
                        &btc_conf.rpc_user,
                        &btc_conf.rpc_pass,
                    );

                    block_processor::run(postgres.clone(), rpc_client.clone(), skip_missed_blocks);
                    payouter::run(postgres.clone(), rpc_client.clone(), btc_config.network);
                }
                Currency::Eth => {
                    use block_processor::ethereum::service as block_processor;
                    use payouter::ethereum::service as payouter;

                    let eth_config = config.clone().ethereum.expect("No ethereum configuration.");

                    let rpc_client = EthRpcClient::new(eth_config.rpc_url);

                    block_processor::run(postgres.clone(), rpc_client.clone(), skip_missed_blocks);
                    payouter::run(postgres.clone(), rpc_client.clone(), eth_config.network);
                }

                _ => panic!(),
            }
        }

        server::run(postgres, config);
    });
}

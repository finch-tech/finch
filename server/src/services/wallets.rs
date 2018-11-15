use futures::future::{Future, IntoFuture};

use config::Config;
use core::db::postgres::PgExecutorAddr;
use core::payment::Payment;
use core::store::Store;
use hd_keyring::{HdKeyring, Wallet};
use services::Error;

pub fn create(
    payment: Payment,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Wallet, Error = Error> {
    Store::find_by_id_with_deleted(payment.store_id, postgres)
        .from_err()
        .and_then(|store| {
            let mut path = store.hd_path;

            let timestamp_nanos = payment.created_at.timestamp_nanos().to_string();
            let second = &timestamp_nanos[..10];
            let nano_second = &timestamp_nanos[10..];

            path.push_str("/");
            path.push_str(second);
            path.push_str("/");
            path.push_str(nano_second);

            let config = Config::new();

            HdKeyring::from_mnemonic(&path, &store.mnemonic, 0, config.btc_network)
                .into_future()
                .from_err()
                .and_then(move |keyring| {
                    keyring
                        .get_wallet_by_index(payment.index as u32)
                        .into_future()
                        .from_err()
                })
        })
}

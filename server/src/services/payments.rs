use futures::future::{Future, IntoFuture};
use uuid::Uuid;

use core::{
    db::postgres::PgExecutorAddr,
    payment::{Payment, PaymentPayload},
    store::Store,
};
use currency_api_client::Client as CurrencyApiClient;
use hd_keyring::HdKeyring;
use services::Error;
use types::{bitcoin::Network as BtcNetwork, currency::Crypto, PaymentStatus};

const BTC_SCALE: i64 = 8;
const ETH_SCALE: i64 = 6;

pub fn create(
    mut payload: PaymentPayload,
    store: &Store,
    postgres: &PgExecutorAddr,
    currency_api_client: CurrencyApiClient,
    btc_network: Option<BtcNetwork>,
) -> impl Future<Item = Payment, Error = Error> {
    let postgres = postgres.clone();
    let store = store.to_owned();

    let index: u32 = 1;

    payload.index = Some(index as i32);
    payload.status = Some(PaymentStatus::Pending);
    payload.set_created_at();

    let mut path = store.hd_path.clone();

    let timestamp_nanos = payload.created_at.unwrap().timestamp_nanos().to_string();
    let second = &timestamp_nanos[..10];
    let nano_second = &timestamp_nanos[10..];

    path.push_str("/");
    path.push_str(second);
    path.push_str("/");
    path.push_str(nano_second);

    HdKeyring::from_mnemonic(
        &path,
        &store.mnemonic.clone(),
        0,
        btc_network.unwrap_or(BtcNetwork::TestNet),
    )
    .into_future()
    .from_err()
    .and_then(move |keyring| keyring.get_wallet_by_index(index).into_future().from_err())
    .and_then(move |wallet| {
        currency_api_client
            .get_rate(&payload.fiat.unwrap(), &payload.crypto.unwrap())
            .from_err()
            .and_then(move |rate| {
                match payload.crypto.unwrap() {
                    Crypto::Btc => {
                        payload.charge =
                            Some(payload.clone().price.unwrap() * rate.with_scale(BTC_SCALE));
                    }
                    Crypto::Eth => {
                        payload.charge =
                            Some(payload.clone().price.unwrap() * rate.with_scale(ETH_SCALE));
                    }
                };

                payload.address = Some(wallet.get_address(&payload.crypto.unwrap()));

                Payment::insert(payload, &postgres).from_err()
            })
    })
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id, postgres).from_err()
}

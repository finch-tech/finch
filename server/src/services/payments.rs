use futures::future::{Future, IntoFuture};
use uuid::Uuid;

use config::Config;
use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use currency_api_client::Client as CurrencyApiClient;
use hd_keyring::HdKeyring;
use services::Error;
use types::{Currency, PaymentStatus};

const BTC_SCALE: i64 = 8;
const ETH_SCALE: i64 = 6;

pub fn create(
    mut payload: PaymentPayload,
    postgres: &PgExecutorAddr,
    currency_api_client: CurrencyApiClient,
) -> impl Future<Item = Payment, Error = Error> {
    let postgres = postgres.clone();

    // TODO: Random index.
    let index: i32 = 1;

    payload.index = Some(index);
    payload.status = Some(PaymentStatus::Pending);
    payload.index = Some(index);

    Payment::insert(payload, &postgres)
        .from_err()
        .and_then(move |payment| {
            payment.store(&postgres).from_err().and_then(move |store| {
                let mut path = store.hd_path.clone();

                let timestamp_nanos = payment.created_at.timestamp_nanos().to_string();
                let second = &timestamp_nanos[..10];
                let nano_second = &timestamp_nanos[10..];

                path.push_str("/");
                path.push_str(second);
                path.push_str("/");
                path.push_str(nano_second);

                let config = Config::new();
                let payment_index = payment.index.clone() as u32;

                HdKeyring::from_mnemonic(&path, &store.mnemonic.clone(), 0, config.btc_network)
                    .into_future()
                    .from_err()
                    .and_then(move |keyring| {
                        keyring
                            .get_wallet_by_index(payment_index)
                            .into_future()
                            .from_err()
                    })
                    .and_then(move |wallet| {
                        let mut payload = PaymentPayload::from(payment.clone());

                        currency_api_client
                            .get_rate(&store.base_currency, &payment.typ)
                            .from_err()
                            .and_then(move |rate| {
                                match payment.typ {
                                    Currency::Btc => {
                                        payload.confirmations_required =
                                            store.btc_confirmations_required;
                                        payload.price = Some(
                                            payment.base_price.clone() * rate.with_scale(BTC_SCALE),
                                        );
                                    }
                                    Currency::Eth => {
                                        payload.confirmations_required =
                                            store.eth_confirmations_required;
                                        payload.price = Some(
                                            payment.base_price.clone() * rate.with_scale(ETH_SCALE),
                                        );
                                    }
                                    _ => panic!("Invalid currency"),
                                };

                                payload.address = Some(wallet.get_address(&payment.typ));

                                Payment::update(payment.id, payload, &postgres).from_err()
                            })
                    })
            })
        })
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id, postgres).from_err()
}

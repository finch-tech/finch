use std::collections::HashSet;
use std::env;
use std::str::FromStr;

use futures::future::Future;
use uuid::Uuid;

use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use currency_api_client::{Api as CurrencyApi, Client as CurrencyApiClient};
use services::{self, Error};
use types::{Currency, PaymentStatus};

const BTC_SCALE: i64 = 8;
const ETH_SCALE: i64 = 6;

pub fn create(
    currencies: HashSet<Currency>,
    mut payload: PaymentPayload,
    postgres: &PgExecutorAddr,
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
            services::wallets::create(payment.clone(), &postgres).and_then(move |wallet| {
                let mut payload = PaymentPayload::from(payment.clone());

                payment.store(&postgres).from_err().and_then(move |store| {
                    let currency_api = CurrencyApi::from_str(
                        &env::var("CURRENCY_API")
                            .expect("CURRENCY_API environment variable must be set."),
                    )
                    .unwrap();

                    let currency_api_key = env::var("CURRENCY_API_KEY")
                        .expect("CURRENCY_API_KEY environment variable must be set.");

                    let currency_api_client =
                        CurrencyApiClient::new(&currency_api, &currency_api_key);

                    let btc_rate = currency_api_client
                        .get_rate(&store.base_currency, &Currency::Btc)
                        .from_err();

                    let eth_rate = currency_api_client
                        .get_rate(&store.base_currency, &Currency::Eth)
                        .from_err();

                    btc_rate
                        .join(eth_rate)
                        .and_then(move |(mut btc_rate, mut eth_rate)| {
                            btc_rate = btc_rate.with_scale(BTC_SCALE);
                            eth_rate = eth_rate.with_scale(ETH_SCALE);

                            for (_, c) in currencies.iter().enumerate() {
                                match c {
                                    Currency::Btc => {
                                        payload.btc_address = Some(wallet.get_btc_address());
                                        payload.btc_price =
                                            Some(payment.price.clone() * btc_rate.clone());
                                    }
                                    Currency::Eth => {
                                        payload.eth_address = Some(wallet.get_eth_address());
                                        payload.eth_price =
                                            Some(payment.price.clone() * eth_rate.clone());
                                    }
                                    _ => panic!(),
                                }
                            }

                            Payment::update_by_id(payment.id, payload, &postgres).from_err()
                        })
                })
            })
        })
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id, postgres).from_err()
}

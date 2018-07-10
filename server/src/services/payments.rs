use std::collections::HashSet;

use futures::future::Future;
use uuid::Uuid;

use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
use currency_api_client::Client as CurrencyApiClient;
use services::{self, Error};
use types::{Currency, Status};

pub fn create(
    currencies: HashSet<Currency>,
    mut payload: PaymentPayload,
    postgres: PgExecutorAddr,
) -> impl Future<Item = Payment, Error = Error> {
    // TODO: Random index.
    let index: i32 = 1;

    payload.index = Some(index);
    payload.status = Some(Status::Pending);
    payload.index = Some(index);

    Payment::insert(payload, postgres.clone())
        .from_err()
        .and_then(|payment| {
            services::wallets::create(payment.clone(), postgres.clone()).and_then(move |wallet| {
                let mut payload = PaymentPayload::from(payment.clone());

                let store = payment.store(postgres.clone()).from_err();
                let item = payment.item(postgres.clone()).from_err();

                store.join(item).and_then(move |(store, item)| {
                    let btc_rate =
                        CurrencyApiClient::new(&store.currency_api, &store.currency_api_key)
                            .get_rate(&store.base_currency, &Currency::Btc)
                            .from_err();

                    let eth_rate =
                        CurrencyApiClient::new(&store.currency_api, &store.currency_api_key)
                            .get_rate(&store.base_currency, &Currency::Eth)
                            .from_err();

                    btc_rate
                        .join(eth_rate)
                        .and_then(move |(btc_rate, eth_rate)| {
                            for (_, c) in currencies.iter().enumerate() {
                                match c {
                                    Currency::Btc => {
                                        payload.btc_address = Some(wallet.get_btc_address());
                                        payload.btc_price =
                                            Some(item.price.clone() * btc_rate.clone());
                                    }
                                    Currency::Eth => {
                                        payload.eth_address = Some(wallet.get_eth_address());
                                        payload.eth_price =
                                            Some(item.price.clone() * eth_rate.clone());
                                    }
                                    _ => panic!(),
                                }
                            }

                            Payment::update_by_id(payment.id, payload, postgres).from_err()
                        })
                })
            })
        })
}

pub fn get(id: Uuid, postgres: PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id.clone(), postgres).from_err()
}

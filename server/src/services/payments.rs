use std::collections::HashSet;

use futures::future::Future;
use uuid::Uuid;

use core::db::postgres::PgExecutorAddr;
use core::payment::{Payment, PaymentPayload};
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

                for (_, c) in currencies.iter().enumerate() {
                    match c {
                        Currency::Btc => payload.btc_address = Some(wallet.get_btc_address()),
                        Currency::Eth => payload.eth_address = Some(wallet.get_eth_address()),
                    }
                }

                Payment::update_by_id(payment.id, payload, postgres).from_err()
            })
        })
}

pub fn get(id: Uuid, postgres: PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id.clone(), postgres).from_err()
}

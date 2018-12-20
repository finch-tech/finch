use chrono::{prelude::*, Duration};
use futures::future::{err, ok, Future, IntoFuture};

use core::{
    app_status::AppStatus, db::postgres::PgExecutorAddr, payment::Payment, voucher::Voucher,
};
use services::Error;
use types::{Currency, PaymentStatus};

pub fn create(
    payment: Payment,
    postgres: &PgExecutorAddr,
) -> Box<Future<Item = String, Error = Error>> {
    let postgres = postgres.clone();

    let store = payment.store(&postgres).from_err();
    let status = AppStatus::find(&postgres).from_err();

    Box::new(
        store
            .join(status)
            .and_then(move |(store, status)| {
                let block_height = match payment.typ {
                    Currency::Btc => status.btc_block_height,
                    Currency::Eth => status.eth_block_height,
                    _ => panic!("Invalid currency"),
                };

                if block_height.is_none() {
                    return err(Error::PaymentNotConfirmed);
                }

                if let None = payment.block_height_required {
                    return err(Error::PaymentNotConfirmed);
                }

                if block_height.unwrap() < payment.block_height_required.unwrap() {
                    return err(Error::PaymentNotConfirmed);
                }

                match payment.status {
                    PaymentStatus::Paid => ok((payment, store)),
                    _ => err(Error::PaymentNotConfirmed),
                }
            })
            .and_then(move |(payment, store)| {
                // Voucher JWT expires in 1 minute.
                Voucher::new(payment, Utc::now() + Duration::minutes(1))
                    .encode(&store.private_key)
                    .into_future()
                    .from_err()
            }),
    )
}

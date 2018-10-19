use chrono::{prelude::*, Duration};
use futures::future::{err, ok, Future, IntoFuture};

use core::app_status::AppStatus;
use core::db::postgres::PgExecutorAddr;
use core::payment::Payment;
use core::voucher::Voucher;
use services::Error;
use types::PaymentStatus;

pub fn create(
    payment: Payment,
    postgres: &PgExecutorAddr,
) -> Box<Future<Item = String, Error = Error>> {
    let postgres = postgres.clone();

    let transaction = payment.transaction(&postgres).from_err();
    let store = payment.store(&postgres).from_err();
    let status = AppStatus::find(&postgres).from_err();

    Box::new(
        transaction
            .join(store)
            .join(status)
            .and_then(move |((transaction, store), status)| {
                if let None = status.eth_block_height {
                    return err(Error::PaymentNotConfirmed);
                }

                if let None = payment.eth_block_height_required {
                    return err(Error::PaymentNotConfirmed);
                }

                if status.eth_block_height.unwrap() < payment.eth_block_height_required.unwrap() {
                    return err(Error::PaymentNotConfirmed);
                }

                match payment.status {
                    PaymentStatus::Paid => ok((payment, transaction, store)),
                    _ => err(Error::PaymentNotConfirmed),
                }
            })
            .and_then(move |(payment, transaction, store)| {
                // Voucher JWT expires in 1 minute.
                Voucher::new(payment, transaction, Utc::now() + Duration::minutes(1))
                    .encode(&store.private_key)
                    .into_future()
                    .from_err()
            }),
    )
}

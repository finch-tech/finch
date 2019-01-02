use chrono::{prelude::*, Duration};
use futures::future::{err, ok, Future, IntoFuture};

use core::{db::postgres::PgExecutorAddr, payment::Payment, voucher::Voucher};
use services::Error;
use types::PaymentStatus;

pub fn create(
    payment: Payment,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = String, Error = Error> {
    let postgres = postgres.clone();

    payment
        .store(&postgres)
        .from_err()
        .and_then(move |store| {
            let zero_confirmation_payment = payment.confirmations_required == 0;

            match payment.status {
                PaymentStatus::Pending => return err(Error::PaymentNotConfirmed),
                PaymentStatus::Paid => {
                    if zero_confirmation_payment {
                        ok((payment, store))
                    } else {
                        return err(Error::PaymentNotConfirmed);
                    }
                }
                PaymentStatus::Confirmed | PaymentStatus::Completed => ok((payment, store)),
                _ => err(Error::PaymentNotConfirmed),
            }
        })
        .and_then(move |(payment, store)| {
            // Voucher JWT expires in 1 minute.
            Voucher::from_payment(payment, Utc::now() + Duration::minutes(1))
                .encode(&store.private_key)
                .into_future()
                .from_err()
        })
}

use std::time::Duration;

use actix::prelude::*;
use actix_web::actix::spawn;
use futures::{future, Future};

use core::app_status::AppStatus;
use core::db::postgres::PgExecutorAddr;
use core::payment::Payment;
use payouter::{Payout, PayouterAddr, Refund};
use types::{PaymentStatus, U256};

pub struct Monitor {
    pub block_height: Option<U256>,
    pub payouter: PayouterAddr,
    pub postgres: PgExecutorAddr,
}

impl Monitor {
    pub fn new(payouter: PayouterAddr, postgres: PgExecutorAddr) -> Self {
        Monitor {
            block_height: None,
            payouter,
            postgres,
        }
    }
}

impl Actor for Monitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::new(10, 0), |monitor, _| {
            let postgres = monitor.postgres.clone();
            let payouter = monitor.payouter.clone();

            spawn(
                AppStatus::find(&postgres)
                    .map_err(|_| ())
                    .and_then(move |status| {
                        // TODO: Skip already processed blocks.

                        println!(
                            "Checking confirmed payments on {}",
                            status.block_height.clone().unwrap()
                        );
                        Payment::find_all_confirmed(status.block_height.unwrap(), &postgres)
                            .map_err(|_| ())
                            .and_then(move |payments| {
                                for (_, payment) in payments.iter().enumerate() {
                                    match payment.status {
                                        PaymentStatus::Paid => {
                                            match payouter.try_send(Payout((*payment).clone())) {
                                                Ok(_) => (),
                                                Err(_) => {
                                                    // TODO: Handle error.
                                                }
                                            };
                                        }
                                        PaymentStatus::InsufficientAmount => {
                                            match payouter.try_send(Refund((*payment).clone())) {
                                                Ok(_) => (),
                                                Err(_) => {
                                                    // TODO: Handle error.
                                                }
                                            };
                                        }
                                        PaymentStatus::Expired => {
                                            match payouter.try_send(Refund((*payment).clone())) {
                                                Ok(_) => (),
                                                Err(_) => {
                                                    // TODO: Handle error.
                                                }
                                            };
                                        }
                                        _ => panic!(
                                            "Found invalid payment status in payout monitor."
                                        ),
                                    }
                                }

                                future::ok(())
                            })
                    })
                    .map(|_| ()),
            )
        });
    }
}

use std::time::Duration;

use actix::fut::{self, wrap_future};
use actix::prelude::*;
use futures::{future, stream, Future, Stream};

use core::app_status::AppStatus;
use core::db::postgres::PgExecutorAddr;
use core::payout::Payout;
use payouter::{PayouterAddr, ProcessPayout};
use types::U128;

use errors::Error;

pub struct Monitor {
    pub payouter: PayouterAddr,
    pub postgres: PgExecutorAddr,
    pub previous_block: Option<U128>,
}

impl Monitor {
    pub fn new(payouter: PayouterAddr, postgres: PgExecutorAddr) -> Self {
        Monitor {
            payouter,
            postgres,
            previous_block: None,
        }
    }
}

impl Actor for Monitor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::new(10, 0), move |monitor, ctx| {
            let address = ctx.address();

            let monitor_process = wrap_future(AppStatus::find(&monitor.postgres))
                .from_err::<Error>()
                .and_then(move |status, m: &mut Monitor, _| {
                    if let Some(block_height) = status.block_height {
                        if let Some(ref previous_block) = m.previous_block {
                            if block_height.clone() == *previous_block {
                                return fut::Either::A(fut::ok(()));
                            }
                        };

                        return fut::Either::B(
                            wrap_future(
                                address
                                    .send(ProcessBlock(block_height.clone()))
                                    .from_err()
                                    .and_then(|res| res.map_err(|e| Error::from(e))),
                            ).and_then(move |_, m: &mut Monitor, _| {
                                m.previous_block = Some(block_height);
                                fut::ok(())
                            }),
                        );
                    };

                    fut::Either::A(fut::ok(()))
                })
                .map_err(|e, _, _| match e {
                    _ => println!("Monitor error: {:?}", e),
                })
                .map(|_, _, _| ());

            ctx.spawn(monitor_process);
        });
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessBlock(U128);

impl Handler<ProcessBlock> for Monitor {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        ProcessBlock(block_number): ProcessBlock,
        _: &mut Self::Context,
    ) -> Self::Result {
        let postgres = self.postgres.clone();
        let payouter = self.payouter.clone();

        println!("Payment check before {}", block_number);

        let process_payouts = Payout::find_all_confirmed(block_number.clone(), &postgres)
            .from_err()
            .map(move |payouts| stream::iter_ok(payouts))
            .flatten_stream()
            .and_then(move |payout| {
                payouter
                    .send(ProcessPayout(payout))
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e)))
            })
            .for_each(move |_| future::ok(()));

        Box::new(process_payouts)
    }
}

use actix::prelude::*;

use core::db::postgres::PgExecutorAddr;

pub type ProcessorAddr = Addr<Processor>;

pub struct Processor {
    pub postgres: PgExecutorAddr,
}

impl Actor for Processor {
    type Context = Context<Self>;
}

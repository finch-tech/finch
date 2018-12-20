use std::ops::Deref;

use actix::prelude::*;
use diesel::{r2d2::{ConnectionManager, Pool}, pg::PgConnection};
use r2d2;

pub type PgExecutorAddr = Addr<PgExecutor>;

pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("DB pool failed.")
}

pub struct PgExecutor(pub PgPool);

impl Actor for PgExecutor {
    type Context = SyncContext<Self>;
}

impl Deref for PgExecutor {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

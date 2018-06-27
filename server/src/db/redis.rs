use std::ops::Deref;

use actix::prelude::*;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use redis;
use serde_json;

use db::Error;

pub type RedisExecutorAddr = Addr<RedisExecutor>;

type RedisPool = r2d2::Pool<RedisConnectionManager>;

pub fn init_pool(url: &str) -> RedisPool {
    let manager = RedisConnectionManager::new(url).unwrap();
    r2d2::Pool::builder()
        .build(manager)
        .expect("Redis pool failed.")
}

pub struct RedisExecutor(pub RedisPool);

impl Actor for RedisExecutor {
    type Context = SyncContext<Self>;
}

impl Deref for RedisExecutor {
    type Target = RedisPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RedisSubscriber {
    client: redis::Client,
}

impl Actor for RedisSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        // TODO: Subscribe
    }
}

impl RedisSubscriber {
    pub fn new(url: &str) -> Self {
        let client = redis::Client::open(url).unwrap();
        RedisSubscriber { client }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Publish {
    pub key: String,
    pub value: String,
}

impl Handler<Publish> for RedisExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Publish, _: &mut Self::Context) -> Self::Result {
        let redis_conn = &self.get()?;

        redis::cmd("PUBLISH")
            .arg(&msg.key)
            .arg(msg.value)
            .execute(&**redis_conn);

        Ok(())
    }
}

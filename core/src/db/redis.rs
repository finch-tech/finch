use std::ops::Deref;

use _redis;
use actix::prelude::*;
use r2d2;
use r2d2_redis::RedisConnectionManager;

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

        _redis::cmd("PUBLISH")
            .arg(&msg.key)
            .arg(msg.value)
            .execute(&**redis_conn);

        Ok(())
    }
}

impl Deref for RedisExecutor {
    type Target = RedisPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type RedisSubscriberAddr = Addr<RedisSubscriber>;

pub struct RedisSubscriber {
    client: _redis::Client,
}

impl Actor for RedisSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        // TODO: Subscribe
    }
}

impl RedisSubscriber {
    pub fn new(url: &str) -> Self {
        let client = _redis::Client::open(url).unwrap();
        RedisSubscriber { client }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Event<'a> {
    pub key: &'a str,
    pub value: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe<'a> {
    pub key: &'a str,
    pub recipient: Recipient<Event<'a>>,
}

impl<'a> Handler<Subscribe<'a>> for RedisSubscriber {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        let mut pubsub = self.client.get_pubsub().unwrap();

        pubsub.subscribe(msg.key).unwrap();

        loop {
            let payload: String = match pubsub.get_message() {
                Ok(msg) => match msg.get_payload() {
                    Ok(payload) => payload,
                    _ => return (),
                },
                _ => return (),
            };

            match msg.recipient.do_send(Event {
                key: msg.key,
                value: payload,
            }) {
                Ok(_) => (),
                Err(_) => {
                    // TODO: Handle error.
                    ()
                }
            }
        }
    }
}

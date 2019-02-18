use actix::prelude::*;
use bigdecimal::BigDecimal;
use futures::future::Future;

use api::Api;
use errors::Error;
use types::currency::{Crypto, Fiat};

pub type CurrencyApiClientAddr = Addr<Client>;

#[derive(Clone)]
pub struct Client {
    api: Api,
    key: String,
}

impl Client {
    pub fn new(api: &Api, key: &str) -> Self {
        Client {
            api: api.to_owned(),
            key: key.to_owned(),
        }
    }
}

impl Actor for Client {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<BigDecimal, Error>")]
pub struct GetRate {
    pub from: Fiat,
    pub to: Crypto,
}

impl Handler<GetRate> for Client {
    type Result = Box<Future<Item = BigDecimal, Error = Error>>;

    fn handle(&mut self, GetRate { from, to }: GetRate, _: &mut Self::Context) -> Self::Result {
        self.api.get_rate(from, to, &self.key)
    }
}

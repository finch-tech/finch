use bigdecimal::BigDecimal;
use futures::future::Future;

use api::Api;
use errors::Error;
use types::currency::{Crypto, Fiat};

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

    pub fn get_rate(
        &self,
        from: &Fiat,
        to: &Crypto,
    ) -> Box<Future<Item = BigDecimal, Error = Error>> {
        self.api.get_rate(from, to, &self.key)
    }
}

use bigdecimal::BigDecimal;
use futures::future::Future;

use api::Api;
use errors::Error;
use types::Currency;

#[derive(Clone)]
pub struct Client {
    api: Api,
    key: String,
}

impl Client {
    pub fn new(api: Api, key: String) -> Self {
        Client { api, key }
    }

    pub fn get_rate(
        &self,
        from: &Currency,
        to: &Currency,
    ) -> Box<Future<Item = BigDecimal, Error = Error>> {
        self.api.get_rate(from, to, &self.key)
    }
}

use bigdecimal::BigDecimal;
use futures::future::Future;

use api::Api;
use errors::Error;
use types::Currency;

pub struct Client<'a> {
    api: &'a Api,
    key: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(api: &'a Api, key: &'a str) -> Self {
        Client { api, key }
    }

    pub fn get_rate(
        &self,
        from: &Currency,
        to: &Currency,
    ) -> Box<Future<Item = BigDecimal, Error = Error>> {
        self.api.get_rate(from, to, self.key)
    }
}

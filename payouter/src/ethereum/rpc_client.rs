use std::str::FromStr;

use actix_web::{client, HttpMessage};
use futures::future::{err, ok, Future};
use serde_json::{self, Value};

use ethereum::Error;
use ethereum::SignedTransaction;

use types::{H160, H256, U128, U256};

pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Client { url }
    }

    pub fn get_gas_price(&self) -> Box<Future<Item = U256, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_gasPrice",
                "params": (),
                "id": 1
            })) {
            Ok(req) => req,
            Err(_) => return Box::new(err(Error::ResponseError)),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => {
                        return err(Error::from(e));
                    }
                };

                match body.get("result") {
                    Some(result) => {
                        let decimal =
                            i64::from_str_radix(&result.as_str().unwrap()[2..], 16).unwrap();
                        ok(U256::from_dec_str(&format!("{}", decimal)).unwrap())
                    }
                    None => err(Error::ResponseError),
                }
            })
        }))
    }

    pub fn get_transaction_count(&self, account: H160) -> Box<Future<Item = U128, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionCount",
                "params": (format!("{:?}", &account.0), "latest"),
                "id": 1
            })) {
            Ok(req) => req,
            Err(_) => return Box::new(err(Error::ResponseError)),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => {
                        return err(Error::from(e));
                    }
                };

                match body.get("result") {
                    Some(result) => {
                        let decimal =
                            i64::from_str_radix(&result.as_str().unwrap()[2..], 16).unwrap();
                        ok(U128::from_dec_str(&format!("{}", decimal)).unwrap())
                    }
                    None => err(Error::ResponseError),
                }
            })
        }))
    }

    pub fn send_raw_transaction(
        &self,
        signed_transaction: SignedTransaction,
    ) -> Box<Future<Item = H256, Error = Error>> {
        let rlp = signed_transaction.rlp_encode();

        let req = match client::ClientRequest::post(&self.url).json(json!({
                "jsonrpc": "2.0",
                "method": "eth_sendRawTransaction",
                "params": vec!(format!("0x{}", &rlp)),
                "id": 1
            })) {
            Ok(req) => req,
            Err(_) => return Box::new(err(Error::ResponseError)),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    Some(result) => ok(H256::from_str(&result.as_str().unwrap()).unwrap()),
                    None => err(Error::ResponseError),
                }
            })
        }))
    }
}
use std::str::FromStr;

use actix_web::{client, HttpMessage};
use base64::encode;
use futures::future::{err, ok, Future};
use serde_json::{self, Value};

use core::bitcoin::{Block, Transaction};
use types::{H256, U128};

use bitcoin::Error;

#[derive(Clone)]
pub struct RpcClient {
    url: String,
    basic_auth: String,
}

impl RpcClient {
    pub fn new(url: String, user: String, password: String) -> Self {
        RpcClient {
            url,
            basic_auth: format!(
                "Basic {}",
                encode(format!("{}:{}", user, password).as_bytes())
            ),
        }
    }

    pub fn get_block_count(&self) -> Box<Future<Item = U128, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getblockcount",
                "params": (),
                "id": "1"
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
                    Some(result) => ok(U128::from_dec_str(&format!("{}", result)).unwrap()),
                    None => err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn get_block(&self, block_hash: H256) -> Box<Future<Item = Block, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getblock",
                "params": [format!("{}", block_hash)],
                "id": "1"
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
                    Some(result) => match serde_json::from_str::<Block>(&format!("{}", result)) {
                        Ok(block) => ok(block),
                        Err(e) => return err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn get_block_hash(&self, block_number: U128) -> Box<Future<Item = H256, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getblockhash",
                "params": [block_number.as_u64()],
                "id": "1"
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
                    Some(result) => {
                        ok(H256::from_str(&format!("{}", &result.as_str().unwrap())).unwrap())
                    }
                    None => err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn get_raw_transaction(
        &self,
        hash: H256,
    ) -> Box<Future<Item = Transaction, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getrawtransaction",
                "params": [format!("{}", hash), 1],
                "id": "1"
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
                    Some(result) => {
                        match serde_json::from_str::<Transaction>(&format!("{}", result)) {
                            Ok(transaction) => ok(transaction),
                            Err(e) => return err(Error::from(e)),
                        }
                    }
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }
}

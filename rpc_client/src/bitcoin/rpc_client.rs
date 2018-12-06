use actix_web::{client, HttpMessage};
use base64::encode;
use futures::future::{err, ok, Future};
use rustc_hex::ToHex;
use serde_json::{self, Value};

use core::bitcoin::{Block, Transaction};
use errors::Error;
use types::{H256, U128};

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
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    // TODO: Use serialization. ex. serde_json::from_str::<U128>()
                    Some(result) => match U128::from_dec_str(&format!("{}", result)) {
                        Ok(block_count) => ok(block_count),
                        Err(e) => err(Error::CustomError(format!("{:?}", e))),
                    },
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
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().limit(4194304).from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    Some(result) => match serde_json::from_str::<Block>(&format!("{}", result)) {
                        Ok(block) => ok(block),
                        Err(e) => err(Error::from(e)),
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
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                if let Some(value) = body.get("error") {
                    match value {
                        Value::Null => (),
                        _ => return err(Error::EmptyResponseError),
                    };
                };

                match body.get("result") {
                    Some(result) => match serde_json::from_str::<H256>(&format!("{}", result)) {
                        Ok(hash) => ok(hash),
                        Err(e) => err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
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
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().limit(4194304).from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    Some(result) => {
                        match serde_json::from_str::<Transaction>(&format!("{}", result)) {
                            Ok(transaction) => ok(transaction),
                            Err(e) => err(Error::from(e)),
                        }
                    }
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn send_raw_transaction(
        &self,
        raw_transaction: Vec<u8>,
    ) -> Box<Future<Item = H256, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "sendrawtransaction",
                "params": [raw_transaction.to_hex()],
                "id": "1"
            })) {
            Ok(req) => req,
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    Some(result) => match serde_json::from_str::<H256>(&format!("{}", result)) {
                        Ok(hash) => ok(hash),
                        Err(e) => err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn estimate_smart_fee(&self, block_n: usize) -> Box<Future<Item = f64, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "estimatesmartfee",
                "params": [block_n],
                "id": "1"
            })) {
            Ok(req) => req,
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => return err(Error::from(e)),
                };

                match body.get("result") {
                    Some(result) => {
                        if let Some(fee) = result.get("feerate") {
                            if let Some(fee) = fee.as_f64() {
                                return ok(fee);
                            }
                        }

                        panic!("Got invalid fee rate.");
                    }
                    None => err(Error::EmptyResponseError),
                }
            })
        }))
    }
}

use actix_web::{client, HttpMessage};
use base64::encode;
use futures::{
    future::{err, ok},
    stream, Future, Stream,
};
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
    pub fn new(url: &str, user: &str, password: &str) -> Self {
        RpcClient {
            url: url.to_owned(),
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
                    Some(result) => match serde_json::from_str::<U128>(&format!("{}", result)) {
                        Ok(block_number) => ok(block_number),
                        Err(e) => return err(Error::from(e)),
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

    pub fn get_block_by_number(
        &self,
        block_number: U128,
    ) -> Box<Future<Item = Block, Error = Error>> {
        let rpc_client = self.clone();

        Box::new(
            self.get_block_hash(block_number)
                .from_err::<Error>()
                .and_then(move |hash| {
                    rpc_client
                        .get_block(hash)
                        .from_err::<Error>()
                        .and_then(move |block| {
                            ok(stream::iter_ok(block.tx_hashes[1..].to_vec().clone()))
                        .flatten_stream()
                        .and_then(move |hash| rpc_client.get_raw_transaction(hash).from_err())
                        .fold(
                            Vec::new(),
                            |mut vec, tx| -> Box<Future<Item = Vec<Transaction>, Error = Error>> {
                                vec.push(tx);
                                Box::new(ok(vec))
                            },
                        )
                        .and_then(move |transactions| {
                            let mut block = block;
                            block.transactions = Some(transactions);
                            ok(block)
                        })
                        })
                }),
        )
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

    pub fn get_raw_mempool(&self) -> Box<Future<Item = Vec<H256>, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getrawmempool",
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
                    Some(result) => match serde_json::from_str::<Vec<H256>>(&format!("{}", result))
                    {
                        Ok(hash) => ok(hash),
                        Err(e) => err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }
}

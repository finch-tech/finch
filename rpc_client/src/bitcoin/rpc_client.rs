use std::str::FromStr;

use actix_web::{client, HttpMessage};
use base64::encode;
use futures::future::{err, ok, Future};
use serde_json::{self, Value};

use types::{H160, H256, U128, U256};

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

    pub fn get_block_hash(&self, block_number: U128) -> Box<Future<Item = H256, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .header("Authorization", format!("{}", self.basic_auth))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "1.0",
                "method": "getblockhash",
                "params": [block_number.0.as_u64()],
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
}

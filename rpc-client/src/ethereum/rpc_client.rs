use std::time::Duration;

use actix::prelude::*;
use actix_web::{client, HttpMessage};
use futures::future::{err, ok, Future};
use serde_json::{self, Value};

use core::ethereum::Block;
use errors::Error;
use ethereum::SignedTransaction;
use types::{H160, H256, U128, U256};

pub type RpcClientAddr = Addr<RpcClient>;

#[derive(Clone)]
pub struct RpcClient {
    url: String,
}

impl Actor for RpcClient {
    type Context = Context<Self>;
}

impl RpcClient {
    pub fn new(url: String) -> Self {
        RpcClient { url }
    }

    pub fn get_balance(&self, account: H160) -> Box<Future<Item = U256, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": (account.hex(), "pending"),
                "id": 1
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
                    Some(result) => match serde_json::from_str::<U256>(&format!("{}", result)) {
                        Ok(balance) => ok(balance),
                        Err(e) => err(Error::from(e)),
                    },
                    None => err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn get_block_number(&self) -> Box<Future<Item = U128, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "params": (),
                "id": 1
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

    pub fn get_pending_block(&self) -> Box<Future<Item = Block, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .timeout(Duration::from_secs(20))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_getBlockByNumber",
                "params": ("pending", true),
                "id": 1
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
                        Err(e) => return err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }

    pub fn get_block_by_number(
        &self,
        block_number: U128,
    ) -> Box<Future<Item = Block, Error = Error>> {
        let req = match client::ClientRequest::post(&self.url)
            .timeout(Duration::from_secs(20))
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "method": "eth_getBlockByNumber",
                "params": (block_number.hex(), true),
                "id": 1
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
                        Err(e) => return err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
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
            Err(e) => return Box::new(err(Error::CustomError(format!("{}", e)))),
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
                    Some(result) => match serde_json::from_str::<U256>(&format!("{}", result)) {
                        Ok(gas_price) => ok(gas_price),
                        Err(e) => err(Error::from(e)),
                    },
                    None => err(Error::EmptyResponseError),
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
                "params": (account.hex(), "latest"),
                "id": 1
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
                        Ok(count) => ok(count),
                        Err(e) => err(Error::from(e)),
                    },
                    None => err(Error::EmptyResponseError),
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
                    Some(result) => match serde_json::from_str::<H256>(&format!("{}", result)) {
                        Ok(hash) => ok(hash),
                        Err(e) => err(Error::from(e)),
                    },
                    None => return err(Error::EmptyResponseError),
                }
            })
        }))
    }
}

#[derive(Message)]
#[rtype(result = "Result<U256, Error>")]
pub struct GetBalance(pub H160);

impl Handler<GetBalance> for RpcClient {
    type Result = Box<Future<Item = U256, Error = Error>>;

    fn handle(&mut self, GetBalance(account): GetBalance, _: &mut Self::Context) -> Self::Result {
        self.get_balance(account)
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct GetBlockNumber;

impl Handler<GetBlockNumber> for RpcClient {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(&mut self, _: GetBlockNumber, _: &mut Self::Context) -> Self::Result {
        self.get_block_number()
    }
}

#[derive(Message)]
#[rtype(result = "Result<Block, Error>")]
pub struct GetPendingBlock;

impl Handler<GetPendingBlock> for RpcClient {
    type Result = Box<Future<Item = Block, Error = Error>>;

    fn handle(&mut self, _: GetPendingBlock, _: &mut Self::Context) -> Self::Result {
        self.get_pending_block()
    }
}

#[derive(Message)]
#[rtype(result = "Result<Block, Error>")]
pub struct GetBlockByNumber(pub U128);

impl Handler<GetBlockByNumber> for RpcClient {
    type Result = Box<Future<Item = Block, Error = Error>>;

    fn handle(
        &mut self,
        GetBlockByNumber(block_number): GetBlockByNumber,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.get_block_by_number(block_number)
    }
}

#[derive(Message)]
#[rtype(result = "Result<U256, Error>")]
pub struct GetGasPrice;

impl Handler<GetGasPrice> for RpcClient {
    type Result = Box<Future<Item = U256, Error = Error>>;

    fn handle(&mut self, _: GetGasPrice, _: &mut Self::Context) -> Self::Result {
        self.get_gas_price()
    }
}

#[derive(Message)]
#[rtype(result = "Result<U128, Error>")]
pub struct GetTransactionCount(pub H160);

impl Handler<GetTransactionCount> for RpcClient {
    type Result = Box<Future<Item = U128, Error = Error>>;

    fn handle(
        &mut self,
        GetTransactionCount(account): GetTransactionCount,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.get_transaction_count(account)
    }
}

#[derive(Message)]
#[rtype(result = "Result<H256, Error>")]
pub struct SendRawTransaction(pub SignedTransaction);

impl Handler<SendRawTransaction> for RpcClient {
    type Result = Box<Future<Item = H256, Error = Error>>;

    fn handle(
        &mut self,
        SendRawTransaction(signed_transaction): SendRawTransaction,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.send_raw_transaction(signed_transaction)
    }
}

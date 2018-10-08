use actix::prelude::*;
use futures::future::{ok, Future, IntoFuture};

use errors::Error;
use ethereum_client::{Client, Transaction};

use core::db::postgres::PgExecutorAddr;
use core::payout::{Payout, PayoutPayload};
use core::store::Store;
use core::transaction::Transaction as _Transaction;
use hd_keyring::{HdKeyring, Wallet};
use types::{H256, PayoutAction, PayoutStatus, U128, U256};

pub type PayouterAddr = Addr<Payouter>;

pub struct Payouter {
    pub postgres: PgExecutorAddr,
    pub ethereum_rpc_url: String,
    pub chain_id: u64,
}

impl Payouter {
    pub fn new(pg_addr: PgExecutorAddr, ethereum_rpc_url: String, chain_id: u64) -> Self {
        Payouter {
            postgres: pg_addr,
            ethereum_rpc_url,
            chain_id,
        }
    }

    pub fn prepare_payout(
        &self,
        payout: Payout,
    ) -> impl Future<Item = (Wallet, _Transaction, Store, U256, U128), Error = Error> {
        let postgres = self.postgres.clone();
        let eth_client = Client::new(self.ethereum_rpc_url.clone());

        let store = payout.store(&postgres).from_err();
        let payment = payout.payment(&postgres).from_err();
        let gas_price = eth_client.get_gas_price().from_err();

        store
            .join3(payment, gas_price)
            .and_then(move |(store, payment, gas_price)| {
                let transaction = payment.transaction(&postgres).from_err();
                let nonce = eth_client
                    .get_transaction_count(payment.clone().eth_address.unwrap())
                    .from_err();

                transaction
                    .join(nonce)
                    .and_then(move |(transaction, nonce)| {
                        let mut path = store.hd_path.clone();
                        let timestamp_nanos = payment.created_at.timestamp_nanos().to_string();
                        let second = &timestamp_nanos[..10];
                        let nano_second = &timestamp_nanos[10..];

                        path.push_str("/");
                        path.push_str(second);
                        path.push_str("/");
                        path.push_str(nano_second);

                        HdKeyring::from_mnemonic(&path, &store.mnemonic.clone(), 0)
                            .into_future()
                            .from_err()
                            .and_then(move |keyring| {
                                keyring
                                    .get_wallet_by_index(payment.index as u32)
                                    .into_future()
                                    .from_err()
                                    .and_then(move |wallet| {
                                        ok((wallet, transaction, store, gas_price, nonce))
                                    })
                            })
                    })
            })
    }

    pub fn payout(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let eth_client = Client::new(self.ethereum_rpc_url.clone());
        let chain_id = self.chain_id.clone();

        self.prepare_payout(payout).and_then(
            move |(wallet, transaction, store, gas_price, nonce)| {
                let value = U256(transaction.value.0 - gas_price.0 * U256::from(21_000).0);

                let raw_transaction = Transaction {
                    nonce,
                    gas_price,
                    gas: U256::from(21_000),
                    to: store.eth_payout_addresses.unwrap()[0].clone(),
                    value,
                    data: b"".to_vec(),
                };

                raw_transaction
                    .sign(wallet.secret_key, chain_id)
                    .into_future()
                    .from_err()
                    .and_then(move |signed_transaction| {
                        eth_client
                            .send_raw_transaction(signed_transaction)
                            .from_err()
                    })
            },
        )
    }

    pub fn refund(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let eth_client = Client::new(self.ethereum_rpc_url.clone());
        let chain_id = self.chain_id.clone();

        self.prepare_payout(payout)
            .and_then(move |(wallet, transaction, _, gas_price, nonce)| {
                let value = U256(transaction.value.0 - gas_price.0 * U256::from(21_000).0);

                let raw_transaction = Transaction {
                    nonce,
                    gas_price,
                    gas: U256::from(21_000),
                    to: transaction.from_address,
                    value,
                    data: b"".to_vec(),
                };

                raw_transaction
                    .sign(wallet.secret_key, chain_id)
                    .into_future()
                    .from_err()
                    .and_then(move |signed_transaction| {
                        eth_client
                            .send_raw_transaction(signed_transaction)
                            .from_err()
                    })
            })
    }
}

impl Actor for Payouter {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessPayout(pub Payout);

impl Handler<ProcessPayout> for Payouter {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(
        &mut self,
        ProcessPayout(payout): ProcessPayout,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let address = ctx.address();

        match payout.action {
            PayoutAction::Payout => Box::new(
                address
                    .send(PayOut(payout))
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e))),
            ),
            PayoutAction::Refund => Box::new(
                address
                    .send(Refund(payout))
                    .from_err()
                    .and_then(|res| res.map_err(|e| Error::from(e))),
            ),
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct PayOut(pub Payout);

impl Handler<PayOut> for Payouter {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, PayOut(payout): PayOut, _: &mut Self::Context) -> Self::Result {
        let postgres = self.postgres.clone();

        Box::new(
            self.payout(payout.clone())
                .from_err()
                .and_then(move |hash| {
                    println!("Paid out {:?}", hash);
                    let mut payload = PayoutPayload::from(payout.clone());
                    payload.transaction_hash = Some(hash);
                    payload.status = Some(PayoutStatus::PaidOut);

                    Payout::update_by_id(payout.id, payload, &postgres)
                        .from_err()
                        .map(|_| ())
                })
                .map(|_| ()),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Refund(pub Payout);

impl Handler<Refund> for Payouter {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, Refund(payout): Refund, _: &mut Self::Context) -> Self::Result {
        let postgres = self.postgres.clone();

        Box::new(
            self.refund(payout.clone())
                .from_err()
                .and_then(move |hash| {
                    println!("Refunded {:?}", hash);
                    let mut payload = PayoutPayload::from(payout.clone());
                    payload.transaction_hash = Some(hash);
                    payload.status = Some(PayoutStatus::Refunded);

                    Payout::update_by_id(payout.id, payload, &postgres)
                        .from_err()
                        .map(|_| ())
                })
                .map(|_| ()),
        )
    }
}

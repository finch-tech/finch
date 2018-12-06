use std::str::FromStr;

use actix::prelude::*;
use futures::future::{self, Future, IntoFuture};

use errors::Error;
use rpc_client::ethereum::{RpcClient, UnsignedTransaction};

use core::db::postgres::PgExecutorAddr;
use core::ethereum::Transaction;
use core::payout::{Payout, PayoutPayload};
use core::store::Store;
use hd_keyring::{HdKeyring, Wallet};
use types::{
    bitcoin::Network as BtcNetwork, ethereum::Network as EthNetwork, PayoutAction, PayoutStatus,
    H160, H256, U128, U256,
};

pub type PayouterAddr = Addr<Payouter>;

pub struct Payouter {
    pub postgres: PgExecutorAddr,
    pub rpc_client: RpcClient,
    pub network: EthNetwork,
}

impl Payouter {
    pub fn new(pg_addr: PgExecutorAddr, rpc_client: RpcClient, network: EthNetwork) -> Self {
        Payouter {
            postgres: pg_addr,
            rpc_client,
            network,
        }
    }

    pub fn prepare_payout(
        &self,
        payout: Payout,
    ) -> impl Future<Item = (Wallet, Transaction, Store, U256, U128), Error = Error> {
        let postgres = self.postgres.clone();
        let rpc_client = self.rpc_client.clone();

        let store = payout.store(&postgres).from_err();
        let payment = payout.payment(&postgres).from_err();
        let gas_price = rpc_client.get_gas_price().from_err();

        store
            .join3(payment, gas_price)
            .and_then(move |(store, payment, gas_price)| {
                let transaction =
                    Transaction::find_by_hash(payment.clone().transaction_hash.unwrap(), &postgres)
                        .from_err();
                let nonce = rpc_client
                    .get_transaction_count(
                        H160::from_str(&payment.clone().address.unwrap()[2..]).unwrap(),
                    )
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

                        HdKeyring::from_mnemonic(
                            &path,
                            &store.mnemonic.clone(),
                            0,
                            // Dummy
                            BtcNetwork::MainNet,
                        )
                        .into_future()
                        .from_err()
                        .and_then(move |keyring| {
                            keyring
                                .get_wallet_by_index(payment.index as u32)
                                .into_future()
                                .from_err()
                                .and_then(move |wallet| {
                                    future::ok((wallet, transaction, store, gas_price, nonce))
                                })
                        })
                    })
            })
    }

    pub fn payout(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let chain_id = self.network.chain_id();
        let rpc_client = self.rpc_client.clone();

        self.prepare_payout(payout).and_then(
            move |(wallet, transaction, store, gas_price, nonce)| {
                if let Some(eth_payout_addresses) = store.eth_payout_addresses {
                    let value = transaction.value - gas_price * U256::from(21_000);

                    let raw_transaction = UnsignedTransaction {
                        nonce,
                        gas_price,
                        gas: U256::from(21_000),
                        to: eth_payout_addresses[0],
                        value,
                        data: b"".to_vec(),
                    };

                    return future::Either::A(
                        raw_transaction
                            .sign(wallet.secret_key, chain_id)
                            .into_future()
                            .from_err()
                            .and_then(move |signed_transaction| {
                                rpc_client
                                    .send_raw_transaction(signed_transaction)
                                    .from_err()
                            }),
                    );
                }

                future::Either::B(future::err(Error::NoPayoutAddress))
            },
        )
    }

    pub fn refund(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let chain_id = self.network.chain_id();
        let rpc_client = self.rpc_client.clone();

        self.prepare_payout(payout)
            .and_then(move |(wallet, transaction, _, gas_price, nonce)| {
                let value = transaction.value - gas_price * U256::from(21_000);

                let raw_transaction = UnsignedTransaction {
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
                        rpc_client
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
        let postgres_a = self.postgres.clone();
        let postgres_b = self.postgres.clone();

        Box::new(
            self.payout(payout)
                .from_err()
                .and_then(move |hash| {
                    println!("Paid out {}", hash.hex());
                    let mut payload = PayoutPayload::from(payout);
                    payload.transaction_hash = Some(Some(hash));
                    payload.status = Some(PayoutStatus::PaidOut);

                    Payout::update(payout.id, payload, &postgres_a).from_err()
                })
                .map(move |_| ())
                .or_else(move |e| -> Box<Future<Item = (), Error = Error>> {
                    match e {
                        // If payout address doesn't exist for the store, change payout object's action to Refund.
                        Error::NoPayoutAddress => {
                            let mut payload = PayoutPayload::from(payout);
                            payload.action = Some(PayoutAction::Refund);

                            Box::new(
                                Payout::update(payout.id, payload, &postgres_b)
                                    .from_err()
                                    .map(move |_| ()),
                            )
                        }
                        _ => Box::new(future::err(e)),
                    }
                }),
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

        Box::new(self.refund(payout).from_err().and_then(move |hash| {
            println!("Refunded {}", hash.hex());
            let mut payload = PayoutPayload::from(payout);
            payload.transaction_hash = Some(Some(hash));
            payload.status = Some(PayoutStatus::Refunded);

            Payout::update(payout.id, payload, &postgres)
                .from_err()
                .map(move |_| ())
        }))
    }
}
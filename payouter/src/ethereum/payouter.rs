use std::str::FromStr;

use actix::prelude::*;
use futures::future::{self, Future, IntoFuture};

use blockchain_api_client::ethereum::{
    BlockchainApiClientAddr, GetGasPrice, GetTransactionCount, SendRawTransaction,
    UnsignedTransaction,
};
use core::{
    db::postgres::PgExecutorAddr,
    ethereum::Transaction,
    payment::PaymentPayload,
    payout::{Payout, PayoutPayload},
    store::Store,
};
use errors::Error;
use hd_keyring::{HdKeyring, Wallet};
use types::{
    bitcoin::Network as BtcNetwork, ethereum::Network as EthNetwork, PaymentStatus, PayoutAction,
    PayoutStatus, H160, H256, U128, U256,
};

pub type PayouterAddr = Addr<Payouter>;

pub struct Payouter {
    pub postgres: PgExecutorAddr,
    pub blockchain_api_client: BlockchainApiClientAddr,
    pub network: EthNetwork,
}

impl Payouter {
    pub fn new(
        pg_addr: PgExecutorAddr,
        blockchain_api_client: BlockchainApiClientAddr,
        network: EthNetwork,
    ) -> Self {
        Payouter {
            postgres: pg_addr,
            blockchain_api_client,
            network,
        }
    }

    pub fn prepare_payout(
        &self,
        payout: Payout,
    ) -> impl Future<Item = (Wallet, Transaction, Store, U256, U128), Error = Error> {
        let postgres = self.postgres.clone();
        let blockchain_api_client = self.blockchain_api_client.clone();

        let store = payout.store(&postgres).from_err();
        let payment = payout.payment(&postgres).from_err();
        let gas_price = blockchain_api_client
            .send(GetGasPrice)
            .from_err()
            .and_then(move |res| res.map_err(|e| Error::from(e)));

        store.join3(payment, gas_price).and_then(
            move |(store, payment, gas_price)| -> Box<
                Future<Item = (Wallet, Transaction, Store, U256, U128), Error = Error>,
            > {
                if gas_price == U256::from(0) {
                    return Box::new(future::err(Error::InvalidGasPrice));
                }

                let transaction =
                    Transaction::find_by_hash(payment.clone().transaction_hash.unwrap(), &postgres)
                        .from_err();

                let nonce = blockchain_api_client
                    .send(GetTransactionCount(
                        H160::from_str(&payment.clone().address[2..]).unwrap(),
                    ))
                    .from_err()
                    .and_then(move |res| res.map_err(|e| Error::from(e)));

                Box::new(transaction.join(nonce).and_then(
                    move |(transaction, nonce)| -> Box<
                        Future<Item = (Wallet, Transaction, Store, U256, U128), Error = Error>,
                    > {
                        if transaction.value <= (gas_price * U256::from(21_000)) {
                            info!("Insufficient funds to pay out");
                            return Box::new(future::err(Error::InsufficientFunds));
                        }

                        let mut path = store.hd_path.clone();
                        let timestamp_nanos = payment.created_at.timestamp_nanos().to_string();
                        let second = &timestamp_nanos[..10];
                        let nano_second = &timestamp_nanos[10..];

                        path.push_str("/");
                        path.push_str(second);
                        path.push_str("/");
                        path.push_str(nano_second);

                        Box::new(
                            HdKeyring::from_mnemonic(
                                &path,
                                &store.mnemonic.clone(),
                                0,
                                // Dummy
                                BtcNetwork::TestNet,
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
                            }),
                        )
                    },
                ))
            },
        )
    }

    pub fn payout(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let chain_id = self.network.chain_id();
        let blockchain_api_client = self.blockchain_api_client.clone();

        self.prepare_payout(payout)
            .and_then(move |(wallet, transaction, store, gas_price, nonce)| {
                match store.eth_payout_addresses {
                    Some(payout_addresses) => {
                        future::ok((wallet, transaction, gas_price, nonce, payout_addresses))
                    }
                    None => future::err(Error::NoPayoutAddress),
                }
            })
            .and_then(
                move |(wallet, transaction, gas_price, nonce, payout_addresses)| {
                    let value = transaction.value - gas_price * U256::from(21_000);

                    let raw_transaction = UnsignedTransaction {
                        nonce,
                        gas_price,
                        gas: U256::from(21_000),
                        to: payout_addresses[0],
                        value,
                        data: b"".to_vec(),
                    };

                    raw_transaction
                        .sign(wallet.secret_key, chain_id)
                        .into_future()
                        .from_err()
                        .and_then(move |signed_transaction| {
                            blockchain_api_client
                                .send(SendRawTransaction(signed_transaction))
                                .from_err()
                                .and_then(move |res| res.map_err(|e| Error::from(e)))
                        })
                },
            )
    }

    pub fn refund(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let chain_id = self.network.chain_id();
        let blockchain_api_client = self.blockchain_api_client.clone();

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

                Box::new(
                    raw_transaction
                        .sign(wallet.secret_key, chain_id)
                        .into_future()
                        .from_err()
                        .and_then(move |signed_transaction| {
                            blockchain_api_client
                                .send(SendRawTransaction(signed_transaction))
                                .from_err()
                                .and_then(move |res| res.map_err(|e| Error::from(e)))
                        }),
                )
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
        let postgres = self.postgres.clone();

        let process: Self::Result = match payout.action {
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
        };

        Box::new(process.or_else(move |e| -> Self::Result {
            match e {
                Error::InsufficientFunds => {
                    let mut payload = PayoutPayload::from(payout);
                    payload.status = Some(PayoutStatus::InsufficientFunds);

                    return Box::new(
                        Payout::update(payout.id, payload, &postgres)
                            .from_err()
                            .map(move |_| ()),
                    );
                }
                _ => Box::new(future::err(e)),
            }
        }))
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct PayOut(pub Payout);

impl Handler<PayOut> for Payouter {
    type Result = Box<Future<Item = (), Error = Error>>;

    fn handle(&mut self, PayOut(payout): PayOut, _: &mut Self::Context) -> Self::Result {
        let postgres = self.postgres.clone();

        Box::new(self.payout(payout).from_err().and_then(move |hash| {
            info!("Paid out {}", hash.hex());

            let mut payout_payload = PayoutPayload::from(payout);
            payout_payload.transaction_hash = Some(Some(hash));
            payout_payload.status = Some(PayoutStatus::PaidOut);

            let mut payment_payload = PaymentPayload::new();
            payment_payload.status = Some(PaymentStatus::Completed);

            Payout::update_with_payment(payout.id, payout_payload, payment_payload, &postgres)
                .from_err()
                .map(move |_| ())
                .or_else(move |e| -> Box<Future<Item = (), Error = Error>> {
                    match e {
                        // If payout address doesn't exist for the store, change payout object's action to Refund.
                        Error::NoPayoutAddress => {
                            let mut payload = PayoutPayload::from(payout);
                            payload.action = Some(PayoutAction::Refund);

                            Box::new(
                                Payout::update(payout.id, payload, &postgres)
                                    .from_err()
                                    .map(move |_| ()),
                            )
                        }
                        _ => Box::new(future::err(e)),
                    }
                })
        }))
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
            info!("Refunded {}", hash.hex());
            let mut payload = PayoutPayload::from(payout);
            payload.transaction_hash = Some(Some(hash));
            payload.status = Some(PayoutStatus::Refunded);

            Payout::update(payout.id, payload, &postgres)
                .from_err()
                .map(move |_| ())
        }))
    }
}

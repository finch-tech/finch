use actix::prelude::*;
use futures::future::{self, Future, IntoFuture};

use errors::Error;
use rpc_client::bitcoin::{RpcClient, UnsignedTransaction};

use config::Config;
use core::{
    bitcoin::{ScriptType, Transaction},
    db::postgres::PgExecutorAddr,
    payment::{Payment, PaymentPayload},
    payout::{Payout, PayoutPayload},
    store::Store,
};
use hd_keyring::{HdKeyring, Wallet};
use types::{bitcoin::Network as BtcNetwork, PaymentStatus, PayoutStatus, H256};

pub type PayouterAddr = Addr<Payouter>;

pub struct Payouter {
    pub postgres: PgExecutorAddr,
    pub rpc_client: RpcClient,
    pub network: BtcNetwork,
}

impl Payouter {
    pub fn new(pg_addr: PgExecutorAddr, rpc_client: RpcClient, network: BtcNetwork) -> Self {
        Payouter {
            postgres: pg_addr,
            rpc_client,
            network,
        }
    }

    pub fn prepare_payout(
        &self,
        payout: Payout,
    ) -> impl Future<Item = (Wallet, Transaction, Store, f64), Error = Error> {
        let config = Config::new();

        let postgres = self.postgres.clone();
        let rpc_client = self.rpc_client.clone();

        let store = payout.store(&postgres).from_err();
        let payment = payout.payment(&postgres).from_err();
        let transaction_fee = rpc_client.estimate_smart_fee(1).from_err();

        store
            .join3(payment, transaction_fee)
            .and_then(move |(store, payment, transaction_fee)| {
                Transaction::find_by_txid(payment.clone().transaction_hash.unwrap(), &postgres)
                    .from_err()
                    .and_then(move |transaction| {
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
                            config.btc_network,
                        )
                        .into_future()
                        .from_err()
                        .and_then(move |keyring| {
                            keyring
                                .get_wallet_by_index(payment.index as u32)
                                .into_future()
                                .from_err()
                                .and_then(move |wallet| {
                                    future::ok((wallet, transaction, store, transaction_fee))
                                })
                        })
                    })
            })
    }

    pub fn payout(&self, payout: Payout) -> impl Future<Item = H256, Error = Error> {
        let rpc_client = self.rpc_client.clone();

        self.prepare_payout(payout).and_then(
            move |(wallet, transaction, store, transaction_fee)| {
                if let Some(payout_addresses) = store.btc_payout_addresses {
                    let recepient = wallet.get_btc_address();

                    let mut utxo_n = 0;
                    for (_, output) in transaction.vout.iter().enumerate() {
                        match output.script.script_type {
                            ScriptType::PubKeyHash => {
                                if let Some(ref addresses) = output.script.addresses {
                                    if addresses[0] == recepient {
                                        utxo_n = output.n;
                                    }
                                }
                            }
                            _ => panic!("Unexpected script type"),
                        };
                    }
                    let utxo = transaction.vout[utxo_n as usize].clone();
                    let mut value = (utxo.value * (100_000_000 as f64)) as u64;

                    // In satoshi
                    let tx_fee_per_byte = (transaction_fee * (100_000_000 as f64)) / 1000 as f64;

                    value -= tx_fee_per_byte as u64 * 192;

                    let mut tx = UnsignedTransaction::new(
                        vec![(transaction.clone(), utxo.n)],
                        vec![(payout_addresses[0].clone(), value)],
                    );

                    tx.sign(wallet.secret_key, wallet.public_key);
                    let raw_transaction = tx.into_raw_transaction();

                    return future::Either::A(
                        rpc_client.send_raw_transaction(raw_transaction).from_err(),
                    );
                }

                future::Either::B(future::err(Error::NoPayoutAddress))
            },
        )
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
        Box::new(
            ctx.address()
                .send(PayOut(payout))
                .from_err()
                .and_then(|res| res.map_err(|e| Error::from(e))),
        )
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
            self.payout(payout)
                .from_err()
                .and_then(move |hash| {
                    println!("Paid out {}", hash);

                    let mut payout_payload = PayoutPayload::from(payout);
                    payout_payload.transaction_hash = Some(Some(hash));
                    payout_payload.status = Some(PayoutStatus::PaidOut);
                    let payout_update =
                        Payout::update(payout.id, payout_payload, &postgres).from_err();

                    let mut payment_payload = PaymentPayload::new();
                    payment_payload.status = Some(PaymentStatus::Completed);
                    let payment_update =
                        Payment::update(payout.payment_id, payment_payload, &postgres).from_err();

                    payout_update.join(payment_update)
                })
                .map(move |_| ()),
        )
    }
}

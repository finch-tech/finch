use actix::prelude::*;
use futures::future::{self, Future, IntoFuture};

use blockchain_api_client::bitcoin::{
    BlockchainApiClientAddr, EstimateSmartFee, SendRawTransaction, UnsignedTransaction,
};
use errors::Error;

use core::{
    bitcoin::{ScriptType, Transaction},
    db::postgres::PgExecutorAddr,
    payment::PaymentPayload,
    payout::{Payout, PayoutPayload},
    store::Store,
};
use hd_keyring::{HdKeyring, Wallet};
use types::{bitcoin::Network as BtcNetwork, PaymentStatus, PayoutStatus, H256};

pub type PayouterAddr = Addr<Payouter>;

pub struct Payouter {
    pub postgres: PgExecutorAddr,
    pub blockchain_api_client: BlockchainApiClientAddr,
    pub network: BtcNetwork,
}

impl Payouter {
    pub fn new(
        pg_addr: PgExecutorAddr,
        blockchain_api_client: BlockchainApiClientAddr,
        network: BtcNetwork,
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
    ) -> impl Future<Item = (Wallet, Transaction, Store, f64), Error = Error> {
        let postgres = self.postgres.clone();
        let blockchain_api_client = self.blockchain_api_client.clone();
        let network = self.network.clone();

        let store = payout.store(&postgres).from_err();
        let payment = payout.payment(&postgres).from_err();
        let transaction_fee = blockchain_api_client
            .send(EstimateSmartFee(10))
            .from_err()
            .and_then(move |res| res.map_err(|e| Error::from(e)))
            .from_err();

        store
            .join3(payment, transaction_fee)
            .and_then(move |(store, payment, transaction_fee)| {
                if transaction_fee == 0 as f64 {
                    return future::err(Error::InvalidGasPrice);
                }

                future::ok((store, payment, transaction_fee))
            })
            .and_then(move |(store, payment, transaction_fee)| {
                Transaction::find_by_hash(payment.clone().transaction_hash.unwrap(), &postgres)
                    .from_err()
                    .and_then(move |transaction| {
                        let mut path = store.hd_path.clone();

                        path.push_str("/");
                        path.push_str(&payment.created_at.timestamp().to_string());
                        path.push_str("/");
                        path.push_str(&payment.created_at.timestamp_subsec_micros().to_string());

                        HdKeyring::from_mnemonic(&path, &store.mnemonic.clone(), 0, network)
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
        let blockchain_api_client = self.blockchain_api_client.clone();

        self.prepare_payout(payout)
            .and_then(
                move |(wallet, transaction, store, transaction_fee)| -> Box<Future<Item=H256, Error = Error>> {
                    let payout_address = if let Some(payout_addresses) = store.btc_payout_addresses {
                        payout_addresses[0].to_owned()
                    } else {
                        return Box::new(future::err(Error::NoPayoutAddress));
                    };

                    let recepient = wallet.get_btc_address();

                    let mut utxo_n = 0;
                    for output in transaction.vout.iter() {
                        match output.script.script_type {
                            ScriptType::PubKeyHash => {
                                if let Some(ref addresses) = output.script.addresses {
                                    if addresses[0] == recepient {
                                        utxo_n = output.n;
                                    }
                                }
                            }
                            _ => (),
                        };
                    }
                    let utxo = transaction.vout[utxo_n as usize].clone();
                    let value = (utxo.value * (100_000_000 as f64)) as u64;

                    // In satoshi
                    let tx_fee_per_byte = (transaction_fee * (100_000_000 as f64)) / 1000 as f64;

                    if value <= tx_fee_per_byte as u64 * 192 {
                        info!("Insufficient funds to pay out.");
                        return Box::new(future::err(Error::InsufficientFunds));
                    }

                    let mut tx = UnsignedTransaction::new(
                        vec![(transaction.clone(), utxo.n)],
                        vec![(payout_address.to_string().clone(), value - tx_fee_per_byte as u64 * 192)],
                    );

                    tx.sign(wallet.secret_key, wallet.public_key);
                    let raw_transaction = tx.into_raw_transaction();

                    Box::new(blockchain_api_client
                        .send(SendRawTransaction(raw_transaction))
                        .from_err()
                        .and_then(move |res| res.map_err(|e| Error::from(e))))
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
        let _postgres = self.postgres.clone();

        Box::new(
            self.payout(payout)
                .from_err()
                .and_then(move |hash| {
                    info!("Paid out {}", hash);

                    let mut payout_payload = PayoutPayload::from(payout);
                    payout_payload.transaction_hash = Some(Some(hash));
                    payout_payload.status = Some(PayoutStatus::PaidOut);

                    let mut payment_payload = PaymentPayload::new();
                    payment_payload.status = Some(PaymentStatus::Completed);

                    Payout::update_with_payment(
                        payout.id,
                        payout_payload,
                        payment_payload,
                        &postgres,
                    )
                    .from_err()
                })
                .map(move |_| ())
                .or_else(move |e| -> Self::Result {
                    match e {
                        Error::InsufficientFunds => {
                            let mut payload = PayoutPayload::from(payout);
                            payload.status = Some(PayoutStatus::InsufficientFunds);

                            return Box::new(
                                Payout::update(payout.id, payload, &_postgres)
                                    .from_err()
                                    .map(move |_| ()),
                            );
                        }
                        _ => Box::new(future::err(e)),
                    }
                }),
        )
    }
}

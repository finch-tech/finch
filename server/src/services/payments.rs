use bigdecimal::BigDecimal;
use futures::future::{self, Future, IntoFuture};
use uuid::Uuid;

use core::{
    db::postgres::PgExecutorAddr,
    payment::{Payment, PaymentPayload},
    store::Store,
};
use currency_api_client::{CurrencyApiClientAddr, GetRate};
use hd_keyring::HdKeyring;
use services::Error;
use types::{bitcoin::Network as BtcNetwork, currency::Crypto, PaymentStatus};

const BTC_SCALE: i64 = 8;
const ETH_SCALE: i64 = 6;

pub fn create(
    mut payload: PaymentPayload,
    store: &Store,
    postgres: &PgExecutorAddr,
    min_charge: Option<BigDecimal>,
    currency_api_client: CurrencyApiClientAddr,
) -> impl Future<Item = Payment, Error = Error> {
    let postgres = postgres.clone();
    let store = store.to_owned();

    let index: u32 = 1;

    payload.index = Some(index as i32);
    payload.status = Some(PaymentStatus::Pending);
    payload.set_created_at();

    let mut path = store.hd_path.clone();

    let created_at = payload.created_at.unwrap();

    path.push_str("/");
    path.push_str(&created_at.timestamp().to_string());
    path.push_str("/");
    path.push_str(&created_at.timestamp_subsec_micros().to_string());

    HdKeyring::from_mnemonic(
        &path,
        &store.mnemonic.clone(),
        0,
        payload.btc_network.unwrap_or(BtcNetwork::Test),
    )
    .into_future()
    .from_err()
    .and_then(move |keyring| keyring.get_wallet_by_index(index).into_future().from_err())
    .and_then(move |wallet| {
        currency_api_client
            .send(GetRate {
                from: payload.fiat.unwrap(),
                to: payload.crypto.unwrap(),
            })
            .from_err()
            .and_then(move |res| res.map_err(|e| Error::from(e)))
            .and_then(move |rate| -> Box<Future<Item = Payment, Error = Error>> {
                let charge = match payload.crypto.unwrap() {
                    Crypto::Btc => payload.clone().price.unwrap() * rate.with_scale(BTC_SCALE),
                    Crypto::Eth => payload.clone().price.unwrap() * rate.with_scale(ETH_SCALE),
                };

                if let Some(min_charge) = min_charge {
                    if charge < min_charge {
                        return Box::new(future::err(Error::ChargeAmountTooLow {
                            min: min_charge,
                            unit: payload.crypto.unwrap(),
                        }));
                    }
                }

                payload.charge = Some(charge);

                payload.address = Some(wallet.get_address(&payload.crypto.unwrap()));

                Box::new(Payment::insert(payload, &postgres).from_err())
            })
    })
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Payment, Error = Error> {
    Payment::find_by_id(id, postgres).from_err()
}

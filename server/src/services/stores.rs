use futures::future::{Future, IntoFuture};
use openssl::rsa::Rsa;
use uuid::Uuid;

use core::db::postgres::PgExecutorAddr;
use core::store::{Store, StorePayload};
use hd_keyring::HdKeyring;
use services::Error;
use types::{PrivateKey, PublicKey};

fn generate_rsa() -> Result<(PrivateKey, PublicKey), Error> {
    let rsa = Rsa::generate(2048)?;
    let private_key = rsa.private_key_to_der()?;
    let public_key = rsa.public_key_to_der_pkcs1()?;

    Ok((private_key, public_key))
}

pub fn create(
    mut payload: StorePayload,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Store, Error = Error> {
    let postgres = postgres.clone();
    let kay_pair = generate_rsa().into_future();
    let keyring = HdKeyring::new("m/44'/60'/0'/0", 1).into_future().from_err();

    kay_pair
        .join(keyring)
        .and_then(move |((private_key, public_key), keyring)| {
            payload.mnemonic = Some(keyring.mnemonic.get_string());
            payload.hd_path = Some(keyring.hd_path.to_string());
            payload.private_key = Some(private_key);
            payload.public_key = Some(public_key);

            Store::insert(payload, &postgres).from_err()
        })
}

pub fn patch(
    id: Uuid,
    payload: StorePayload,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Store, Error = Error> {
    let postgres = postgres.clone();

    Store::update(id, payload, &postgres).from_err()
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Store, Error = Error> {
    Store::find_by_id(id, postgres).from_err()
}

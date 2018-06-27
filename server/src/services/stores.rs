use futures::future::{Future, IntoFuture};
use openssl::rsa::Rsa;
use uuid::Uuid;

use db::postgres::PgExecutorAddr;
use models::store::{Store, StorePayload};
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
    postgres: PgExecutorAddr,
) -> impl Future<Item = Store, Error = Error> {
    generate_rsa()
        .into_future()
        .and_then(move |(private_key, public_key)| {
            payload.private_key = Some(private_key);
            payload.public_key = Some(public_key);
            payload.active = Some(true);

            Store::insert(payload, postgres).from_err()
        })
}

pub fn get(id: Uuid, postgres: PgExecutorAddr) -> impl Future<Item = Store, Error = Error> {
    Store::find_by_id(id.clone(), postgres).from_err()
}

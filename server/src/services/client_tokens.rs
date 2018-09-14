use futures::future::Future;
use uuid::Uuid;

use core::client_token::{ClientToken, ClientTokenPayload};
use core::db::postgres::PgExecutorAddr;
use services::Error;

pub fn create(
    payload: ClientTokenPayload,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::insert(payload, postgres).from_err()
}

pub fn get_by_token_and_domain(
    token: Uuid,
    domain: String,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::find_by_token_and_domain(token, domain, postgres).from_err()
}

pub fn find_by_store(
    store_id: Uuid,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Vec<ClientToken>, Error = Error> {
    ClientToken::find_by_store(store_id, postgres).from_err()
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::find_by_id(id, postgres).from_err()
}

pub fn delete(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = usize, Error = Error> {
    ClientToken::delete(id, postgres).from_err()
}

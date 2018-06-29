use futures::future::Future;
use uuid::Uuid;

use db::postgres::PgExecutorAddr;
use models::client_token::{ClientToken, ClientTokenPayload};
use services::Error;

pub fn create(
    payload: ClientTokenPayload,
    postgres: PgExecutorAddr,
) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::insert(payload, postgres).from_err()
}

pub fn get(id: Uuid, postgres: PgExecutorAddr) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::find_by_id(id.clone(), postgres).from_err()
}

pub fn get_by_token(
    token: Uuid,
    postgres: PgExecutorAddr,
) -> impl Future<Item = ClientToken, Error = Error> {
    ClientToken::find_by_token(token.clone(), postgres).from_err()
}

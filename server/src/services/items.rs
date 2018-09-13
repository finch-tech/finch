use futures::future::Future;
use uuid::Uuid;

use core::db::postgres::PgExecutorAddr;
use core::item::{Item, ItemPayload};
use services::Error;

pub fn create(
    payload: ItemPayload,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Item, Error = Error> {
    Item::insert(payload, postgres).from_err()
}

pub fn patch(
    id: Uuid,
    payload: ItemPayload,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = Item, Error = Error> {
    let postgres = postgres.clone();

    Item::update(id, payload, &postgres).from_err()
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = Item, Error = Error> {
    Item::find_by_id(id, postgres).from_err()
}

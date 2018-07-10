use futures::future::Future;

use core::db::postgres::PgExecutorAddr;
use core::item::{Item, ItemPayload};
use services::Error;

pub fn create(
    payload: ItemPayload,
    postgres: PgExecutorAddr,
) -> impl Future<Item = Item, Error = Error> {
    Item::insert(payload, postgres).from_err()
}

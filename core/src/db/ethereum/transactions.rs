use actix::prelude::*;
use diesel::prelude::*;

use db::{
    postgres::{PgExecutor, PooledConnection},
    Error,
};
use models::ethereum::Transaction;
use types::H256;

pub fn insert(payload: Transaction, conn: &PooledConnection) -> Result<Transaction, Error> {
    use diesel::insert_into;
    use schema::eth_transactions::dsl::*;

    insert_into(eth_transactions)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_hash(transaction_hash: H256, conn: &PooledConnection) -> Result<Transaction, Error> {
    use schema::eth_transactions::dsl::*;

    eth_transactions
        .filter(hash.eq(transaction_hash))
        .first::<Transaction>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, Error>")]
pub struct Insert(pub Transaction);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Transaction, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, Error>")]
pub struct FindByHash(pub H256);

impl Handler<FindByHash> for PgExecutor {
    type Result = Result<Transaction, Error>;

    fn handle(
        &mut self,
        FindByHash(transaction_hash): FindByHash,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_by_hash(transaction_hash, &conn)
    }
}

use actix::prelude::*;
use diesel::prelude::*;

use db::{postgres::PgExecutor, Error};
use models::ethereum::Transaction;
use types::H256;

#[derive(Message)]
#[rtype(result = "Result<Transaction, Error>")]
pub struct Insert(pub Transaction);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Transaction, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::eth_transactions::dsl::*;

        let conn = &self.get()?;

        insert_into(eth_transactions)
            .values(&payload)
            .get_result(conn)
            .map_err(|e| Error::from(e))
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
        use schema::eth_transactions::dsl::*;

        let conn = &self.get()?;

        eth_transactions
            .filter(hash.eq(transaction_hash))
            .first::<Transaction>(conn)
            .map_err(|e| Error::from(e))
    }
}

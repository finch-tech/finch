use actix::prelude::*;
use diesel::prelude::*;
use serde_json::{self, Value};

use db::{
    postgres::{PgExecutor, PooledConnection},
    Error,
};
use models::bitcoin::Transaction;
use schema::btc_transactions;
use types::H256;

#[derive(Insertable, Queryable)]
struct BtcTransaction {
    hash: H256,
    data: Value,
}

pub fn insert(payload: Transaction, conn: &PooledConnection) -> Result<Transaction, Error> {
    use diesel::insert_into;
    use schema::btc_transactions::dsl;

    let tx = BtcTransaction {
        hash: payload.hash,
        data: json!(payload),
    };

    let transaction = insert_into(dsl::btc_transactions)
        .values(&tx)
        .get_result::<BtcTransaction>(conn)
        .map_err(|e| Error::from(e))?;

    serde_json::from_str::<Transaction>(&format!("{}", transaction.data))
        .map_err(|e| Error::from(e))
}

pub fn find_by_hash(hash: H256, conn: &PooledConnection) -> Result<Transaction, Error> {
    use schema::btc_transactions::dsl;

    let transaction = dsl::btc_transactions
        .filter(dsl::hash.eq(hash))
        .first::<BtcTransaction>(conn)
        .map_err(|e| Error::from(e))?;

    serde_json::from_str::<Transaction>(&format!("{}", transaction.data))
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

    fn handle(&mut self, FindByHash(hash): FindByHash, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_hash(hash, &conn)
    }
}

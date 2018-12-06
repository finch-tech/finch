use actix::prelude::*;
use diesel::prelude::*;
use serde_json::{self, Value};

use db::{postgres::PgExecutor, Error};
use models::bitcoin::Transaction;
use schema::btc_transactions;
use types::H256;

#[derive(Insertable, Queryable)]
struct BtcTransaction {
    txid: H256,
    data: Value,
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, Error>")]
pub struct Insert(pub Transaction);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Transaction, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::btc_transactions::dsl;

        let conn = &self.get()?;

        let tx = BtcTransaction {
            txid: payload.txid,
            data: json!(payload),
        };

        let transaction = insert_into(dsl::btc_transactions)
            .values(&tx)
            .get_result::<BtcTransaction>(conn)
            .map_err(|e| Error::from(e))?;

        serde_json::from_str::<Transaction>(&format!("{}", transaction.data))
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, Error>")]
pub struct FindByTxId(pub H256);

impl Handler<FindByTxId> for PgExecutor {
    type Result = Result<Transaction, Error>;

    fn handle(&mut self, FindByTxId(txid): FindByTxId, _: &mut Self::Context) -> Self::Result {
        use schema::btc_transactions::dsl;

        let conn = &self.get()?;

        let transaction = dsl::btc_transactions
            .filter(dsl::txid.eq(txid))
            .first::<BtcTransaction>(conn)
            .map_err(|e| Error::from(e))?;

        serde_json::from_str::<Transaction>(&format!("{}", transaction.data))
            .map_err(|e| Error::from(e))
    }
}

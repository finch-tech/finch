use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::PgExecutor;
use db::Error;
use models::store::{Store, StorePayload};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct Insert(pub StorePayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::stores::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(stores)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(&mut self, FindById(store_id): FindById, _: &mut Self::Context) -> Self::Result {
        use schema::stores::dsl::*;

        let pg_conn = &self.get()?;

        stores
            .filter(id.eq(store_id))
            .filter(active.ne(false))
            .first::<Store>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

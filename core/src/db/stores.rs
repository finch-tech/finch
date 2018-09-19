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
pub struct Update {
    pub store_id: Uuid,
    pub payload: StorePayload,
}

impl Handler<Update> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(
        &mut self,
        Update { store_id, payload }: Update,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::stores::dsl::*;

        let pg_conn = &self.get()?;

        update(stores.filter(id.eq(store_id)))
            .set(&payload)
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

#[derive(Message)]
#[rtype(result = "Result<Vec<Store>, Error>")]
pub struct FindByOwner {
    pub owner_id: Uuid,
    pub limit: i64,
    pub offset: i64,
}

impl Handler<FindByOwner> for PgExecutor {
    type Result = Result<Vec<Store>, Error>;

    fn handle(
        &mut self,
        FindByOwner {
            owner_id,
            limit,
            offset,
        }: FindByOwner,
        _: &mut Self::Context,
    ) -> Self::Result {
        let _owner_id = owner_id;
        use schema::stores::dsl::*;

        let pg_conn = &self.get()?;

        stores
            .filter(owner_id.eq(_owner_id).and(active.ne(false)))
            .limit(limit)
            .offset(offset)
            .load::<Store>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(store_id): Delete, _: &mut Self::Context) -> Self::Result {
        use diesel::delete;
        use schema::stores::dsl::*;

        let pg_conn = &self.get()?;

        delete(stores.filter(id.eq(store_id)))
            .execute(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

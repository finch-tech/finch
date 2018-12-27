use actix::prelude::*;
use diesel::prelude::*;

use db::{
    client_tokens,
    {
        postgres::{PgExecutor, PooledConnection},
        Error,
    },
};
use models::store::{Store, StorePayload};
use uuid::Uuid;

pub fn insert(payload: StorePayload, conn: &PooledConnection) -> Result<Store, Error> {
    use diesel::insert_into;
    use schema::stores::dsl;

    insert_into(dsl::stores)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(id: Uuid, payload: StorePayload, conn: &PooledConnection) -> Result<Store, Error> {
    use diesel::update;
    use schema::stores::dsl;

    update(dsl::stores.filter(dsl::id.eq(id).and(dsl::deleted_at.is_null())))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id(id: Uuid, conn: &PooledConnection) -> Result<Store, Error> {
    use schema::stores::dsl;

    dsl::stores
        .filter(dsl::id.eq(id).and(dsl::deleted_at.is_null()))
        .first::<Store>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id_with_deleted(id: Uuid, conn: &PooledConnection) -> Result<Store, Error> {
    use schema::stores::dsl;

    dsl::stores
        .filter(dsl::id.eq(id))
        .first::<Store>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_owner(
    owner_id: Uuid,
    limit: i64,
    offset: i64,
    conn: &PooledConnection,
) -> Result<Vec<Store>, Error> {
    use schema::stores::dsl;

    dsl::stores
        .filter(dsl::owner_id.eq(owner_id).and(dsl::deleted_at.is_null()))
        .limit(limit)
        .offset(offset)
        .load::<Store>(conn)
        .map_err(|e| Error::from(e))
}

pub fn delete(id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::delete;
    use schema::stores::dsl;

    delete(dsl::stores.filter(dsl::id.eq(id).and(dsl::deleted_at.is_null())))
        .execute(conn)
        .map_err(|e| Error::from(e))
}

pub fn soft_delete(id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::update;
    use schema::stores::dsl;

    let mut payload = StorePayload::new();
    payload.set_deleted();

    update(dsl::stores.filter(dsl::id.eq(id).and(dsl::deleted_at.is_null())))
        .set(&payload)
        .get_result::<Store>(conn)
        .map_err(|e| Error::from(e))?;

    client_tokens::delete_by_store_id(id, conn)?;

    Ok(1)
}

pub fn soft_delete_by_owner_id(owner_id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::update;
    use schema::stores::dsl;

    let mut payload = StorePayload::new();
    payload.set_deleted();

    let deleted_stores =
        update(dsl::stores.filter(dsl::owner_id.eq(owner_id).and(dsl::deleted_at.is_null())))
            .set(&payload)
            .get_results::<Store>(conn)?;

    for store in deleted_stores {
        client_tokens::delete_by_store_id(store.id, conn)?;
    }

    Ok(1)
}

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct Insert(pub StorePayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct Update {
    pub id: Uuid,
    pub payload: StorePayload,
}

impl Handler<Update> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(&mut self, Update { id, payload }: Update, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        update(id, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(&mut self, FindById(id): FindById, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_id(id, conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Store, Error>")]
pub struct FindByIdWithDeleted(pub Uuid);

impl Handler<FindByIdWithDeleted> for PgExecutor {
    type Result = Result<Store, Error>;

    fn handle(
        &mut self,
        FindByIdWithDeleted(id): FindByIdWithDeleted,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_by_id_with_deleted(id, &conn)
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
        let conn = &self.get()?;

        find_by_owner(owner_id, limit, offset, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct SoftDelete(pub Uuid);

impl Handler<SoftDelete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, SoftDelete(id): SoftDelete, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        conn.transaction::<_, Error, _>(|| soft_delete(id, &conn))
    }
}

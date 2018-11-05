use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::{PgExecutor, PooledConnection};
use db::Error;
use models::client_token::{ClientToken, ClientTokenPayload};
use uuid::Uuid;

pub fn insert(payload: ClientTokenPayload, conn: &PooledConnection) -> Result<ClientToken, Error> {
    use diesel::insert_into;
    use schema::client_tokens::dsl;

    insert_into(dsl::client_tokens)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id(id: Uuid, conn: &PooledConnection) -> Result<ClientToken, Error> {
    use schema::client_tokens::dsl;

    dsl::client_tokens
        .filter(dsl::id.eq(id))
        .first::<ClientToken>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_token_and_domain(
    token: Uuid,
    domain: String,
    conn: &PooledConnection,
) -> Result<ClientToken, Error> {
    use schema::client_tokens::dsl;

    dsl::client_tokens
        .filter(dsl::token.eq(token).and(dsl::domain.eq(domain)))
        .first::<ClientToken>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_store(
    store_id: Uuid,
    limit: i64,
    offset: i64,
    conn: &PooledConnection,
) -> Result<Vec<ClientToken>, Error> {
    use schema::client_tokens::dsl;

    dsl::client_tokens
        .filter(dsl::store_id.eq(store_id))
        .limit(limit)
        .offset(offset)
        .load::<ClientToken>(conn)
        .map_err(|e| Error::from(e))
}

pub fn delete(id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::delete;
    use schema::client_tokens::dsl;

    delete(dsl::client_tokens.filter(dsl::id.eq(id)))
        .execute(conn)
        .map_err(|e| Error::from(e))
}

pub fn delete_by_store_id(store_id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::delete;
    use schema::client_tokens::dsl;

    delete(dsl::client_tokens.filter(dsl::store_id.eq(store_id)))
        .execute(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct Insert(pub ClientTokenPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(&mut self, FindById(id): FindById, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_id(id, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct FindByTokenAndDomain {
    pub token: Uuid,
    pub domain: String,
}

impl Handler<FindByTokenAndDomain> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(
        &mut self,
        FindByTokenAndDomain { token, domain }: FindByTokenAndDomain,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_by_token_and_domain(token, domain, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<ClientToken>, Error>")]
pub struct FindByStore {
    pub store_id: Uuid,
    pub limit: i64,
    pub offset: i64,
}

impl Handler<FindByStore> for PgExecutor {
    type Result = Result<Vec<ClientToken>, Error>;

    fn handle(
        &mut self,
        FindByStore {
            store_id,
            limit,
            offset,
        }: FindByStore,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_by_store(store_id, limit, offset, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(id): Delete, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        delete(id, &conn)
    }
}

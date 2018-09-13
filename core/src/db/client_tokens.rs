use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::PgExecutor;
use db::Error;
use models::client_token::{ClientToken, ClientTokenPayload};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct Insert(pub ClientTokenPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::client_tokens::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(client_tokens)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(
        &mut self,
        FindById(client_token_id): FindById,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::client_tokens::dsl::*;

        let pg_conn = &self.get()?;

        client_tokens
            .filter(id.eq(client_token_id))
            .first::<ClientToken>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<ClientToken, Error>")]
pub struct FindByToken(pub Uuid);

impl Handler<FindByToken> for PgExecutor {
    type Result = Result<ClientToken, Error>;

    fn handle(
        &mut self,
        FindByToken(client_token): FindByToken,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::client_tokens::dsl::*;

        let pg_conn = &self.get()?;

        client_tokens
            .filter(token.eq(client_token))
            .first::<ClientToken>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<ClientToken>, Error>")]
pub struct FindByStore(pub Uuid);

impl Handler<FindByStore> for PgExecutor {
    type Result = Result<Vec<ClientToken>, Error>;

    fn handle(
        &mut self,
        FindByStore(store_id_query): FindByStore,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::client_tokens::dsl::*;

        let pg_conn = &self.get()?;

        client_tokens
            .filter(store_id.eq(store_id_query))
            .load::<ClientToken>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(client_token_id): Delete, _: &mut Self::Context) -> Self::Result {
        use diesel::delete;
        use schema::client_tokens::dsl::*;

        let pg_conn = &self.get()?;

        delete(client_tokens.filter(id.eq(client_token_id)))
            .execute(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

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

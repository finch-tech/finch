use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::postgres::PgExecutor;
use db::Error;
use models::user::{User, UserPayload};

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct Insert(pub UserPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(users)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct FindByEmail(pub String);

impl Handler<FindByEmail> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(
        &mut self,
        FindByEmail(user_email): FindByEmail,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        users
            .filter(email.eq(user_email))
            .filter(active.ne(false))
            .first::<User>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, FindById(user_id): FindById, _: &mut Self::Context) -> Self::Result {
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        users
            .filter(id.eq(user_id))
            .filter(active.ne(false))
            .first::<User>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

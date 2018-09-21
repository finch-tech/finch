use actix::prelude::*;
use chrono::prelude::*;
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
pub struct Update {
    pub user_id: Uuid,
    pub payload: UserPayload,
}

impl Handler<Update> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(
        &mut self,
        Update { user_id, payload }: Update,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        update(users.filter(id.eq(user_id)))
            .set(&payload)
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
            .filter(
                email
                    .eq(user_email)
                    .and(active.ne(false))
                    .and(is_verified.ne(false)),
            )
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
            .filter(
                id.eq(user_id)
                    .and(active.ne(false))
                    .and(is_verified.ne(false)),
            )
            .first::<User>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct FindByResetToken(pub Uuid);

impl Handler<FindByResetToken> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(
        &mut self,
        FindByResetToken(token): FindByResetToken,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        users
            .filter(
                reset_token
                    .eq(token)
                    .and(reset_token_expires_at.gt(Utc::now())),
            )
            .first::<User>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct Activate(pub Uuid);

impl Handler<Activate> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, Activate(token): Activate, _: &mut Self::Context) -> Self::Result {
        use diesel::update;
        use schema::users::dsl::*;

        let payload = UserPayload {
            email: None,
            password: None,
            salt: None,
            created_at: None,
            updated_at: None,
            is_verified: Some(true),
            verification_token: None,
            verification_token_expires_at: None,
            reset_token: None,
            reset_token_expires_at: None,
            active: None,
        };

        let pg_conn = &self.get()?;

        update(
            users.filter(
                is_verified
                    .ne(true)
                    .and(verification_token.eq(token))
                    .and(verification_token_expires_at.gt(Utc::now())),
            ),
        ).set(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(user_id): Delete, _: &mut Self::Context) -> Self::Result {
        use diesel::delete;
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        delete(users.filter(id.eq(user_id)))
            .execute(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct DeleteExpired(pub String);

impl Handler<DeleteExpired> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(
        &mut self,
        DeleteExpired(user_email): DeleteExpired,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::delete;
        use schema::users::dsl::*;

        let pg_conn = &self.get()?;

        delete(
            users.filter(
                email
                    .eq(user_email)
                    .and(is_verified.ne(true))
                    .and(verification_token_expires_at.lt(Utc::now())),
            ),
        ).execute(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

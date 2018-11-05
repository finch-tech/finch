use actix::prelude::*;
use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::postgres::{PgExecutor, PooledConnection};
use db::stores;
use db::Error;
use models::user::{User, UserPayload};

pub fn insert(payload: UserPayload, conn: &PooledConnection) -> Result<User, Error> {
    use diesel::insert_into;
    use schema::users::dsl;

    insert_into(dsl::users)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(id: Uuid, payload: UserPayload, conn: &PooledConnection) -> Result<User, Error> {
    use diesel::update;
    use schema::users::dsl;

    update(dsl::users.filter(dsl::id.eq(id)))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_email(email: String, conn: &PooledConnection) -> Result<User, Error> {
    use schema::users::dsl;

    dsl::users
        .filter(dsl::email.eq(email).and(dsl::is_verified.ne(false)))
        .first::<User>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id(id: Uuid, conn: &PooledConnection) -> Result<User, Error> {
    use schema::users::dsl;

    dsl::users
        .filter(dsl::id.eq(id).and(dsl::is_verified.ne(false)))
        .first::<User>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_reset_token(token: Uuid, conn: &PooledConnection) -> Result<User, Error> {
    use schema::users::dsl;

    dsl::users
        .filter(
            dsl::reset_token
                .eq(token)
                .and(dsl::reset_token_expires_at.gt(Utc::now())),
        )
        .first::<User>(conn)
        .map_err(|e| Error::from(e))
}

pub fn activate(token: Uuid, conn: &PooledConnection) -> Result<User, Error> {
    use diesel::update;
    use schema::users::dsl;

    let mut payload = UserPayload::new();
    payload.is_verified = Some(true);

    update(
        dsl::users.filter(
            dsl::is_verified
                .ne(true)
                .and(dsl::verification_token.eq(token))
                .and(dsl::verification_token_expires_at.gt(Utc::now())),
        ),
    )
    .set(&payload)
    .get_result(conn)
    .map_err(|e| Error::from(e))
}

pub fn delete(id: Uuid, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::delete;
    use schema::users::dsl;

    delete(dsl::users.filter(dsl::id.eq(id)))
        .execute(conn)
        .map_err(|e| Error::from(e))?;

    stores::soft_delete_by_owner_id(id, &conn)?;

    Ok(1)
}

pub fn delete_expired(email: String, conn: &PooledConnection) -> Result<usize, Error> {
    use diesel::delete;
    use schema::users::dsl;

    delete(
        dsl::users.filter(
            dsl::email
                .eq(email)
                .and(dsl::is_verified.ne(true))
                .and(dsl::verification_token_expires_at.lt(Utc::now())),
        ),
    )
    .execute(conn)
    .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct Insert(pub UserPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct Update {
    pub id: Uuid,
    pub payload: UserPayload,
}

impl Handler<Update> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, Update { id, payload }: Update, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        update(id, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct FindByEmail(pub String);

impl Handler<FindByEmail> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, FindByEmail(email): FindByEmail, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_email(email, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, FindById(id): FindById, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_id(id, &conn)
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
        let conn = &self.get()?;

        find_by_reset_token(token, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<User, Error>")]
pub struct Activate(pub Uuid);

impl Handler<Activate> for PgExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, Activate(token): Activate, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        activate(token, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(id): Delete, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        conn.transaction::<_, Error, _>(|| delete(id, &conn))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct DeleteExpired(pub String);

impl Handler<DeleteExpired> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(
        &mut self,
        DeleteExpired(email): DeleteExpired,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        delete_expired(email, &conn)
    }
}

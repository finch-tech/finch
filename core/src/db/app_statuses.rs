use actix::prelude::*;
use diesel::prelude::*;

use db::{
    postgres::{PgExecutor, PooledConnection},
    Error,
};
use models::app_status::{AppStatus, AppStatusPayload};

pub fn insert(payload: AppStatusPayload, conn: &PooledConnection) -> Result<AppStatus, Error> {
    use diesel::insert_into;
    use schema::app_statuses::dsl;

    insert_into(dsl::app_statuses)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(
    id: i16,
    payload: AppStatusPayload,
    conn: &PooledConnection,
) -> Result<AppStatus, Error> {
    use diesel::update;
    use schema::app_statuses::dsl;

    update(dsl::app_statuses.filter(dsl::id.eq(id)))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id(id: i16, conn: &PooledConnection) -> Result<AppStatus, Error> {
    use schema::app_statuses::dsl;

    dsl::app_statuses
        .filter(dsl::id.eq(id))
        .first::<AppStatus>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct Insert(pub AppStatusPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct Update(pub i16, pub AppStatusPayload);

impl Handler<Update> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(&mut self, Update(id, payload): Update, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        update(id, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct FindById(pub i16);

impl Handler<FindById> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(&mut self, FindById(id): FindById, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_id(id, &conn)
    }
}

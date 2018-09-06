use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::PgExecutor;
use db::Error;
use models::app_status::{AppStatus, AppStatusPayload};

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct Insert(pub AppStatusPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::app_statuses::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(app_statuses)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct UpdateById(pub i16, pub AppStatusPayload);

impl Handler<UpdateById> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(
        &mut self,
        UpdateById(app_status_id, payload): UpdateById,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::app_statuses::dsl::*;

        let pg_conn = &self.get()?;

        update(app_statuses.filter(id.eq(app_status_id)))
            .set(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<AppStatus, Error>")]
pub struct FindById(pub i16);

impl Handler<FindById> for PgExecutor {
    type Result = Result<AppStatus, Error>;

    fn handle(&mut self, FindById(app_status_id): FindById, _: &mut Self::Context) -> Self::Result {
        use schema::app_statuses::dsl::*;

        let pg_conn = &self.get()?;

        app_statuses
            .filter(id.eq(app_status_id))
            .first::<AppStatus>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

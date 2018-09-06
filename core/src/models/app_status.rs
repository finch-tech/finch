use futures::Future;

use db::app_statuses::{FindById, Insert, UpdateById};
use db::postgres::PgExecutorAddr;
use models::Error;
use schema::app_statuses;
use types::U128;

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name = "app_statuses"]
pub struct AppStatusPayload {
    pub id: i16,
    pub block_height: Option<U128>,
}

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "app_statuses"]
pub struct AppStatus {
    pub id: i16,
    pub block_height: Option<U128>,
}

impl AppStatus {
    pub fn insert(
        mut payload: AppStatusPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = AppStatus, Error = Error> {
        payload.id = 1;

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find(postgres: &PgExecutorAddr) -> impl Future<Item = AppStatus, Error = Error> {
        (*postgres)
            .send(FindById(1))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        payload: AppStatusPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = AppStatus, Error = Error> {
        (*postgres)
            .send(UpdateById(1, payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }
}

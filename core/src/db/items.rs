use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::postgres::PgExecutor;
use db::Error;
use models::item::{Item, ItemPayload};

#[derive(Message)]
#[rtype(result = "Result<Item, Error>")]
pub struct Insert(pub ItemPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Item, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::items::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(items)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Item, Error>")]
pub struct Update {
    pub item_id: Uuid,
    pub payload: ItemPayload,
}

impl Handler<Update> for PgExecutor {
    type Result = Result<Item, Error>;

    fn handle(
        &mut self,
        Update { item_id, payload }: Update,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::items::dsl::*;

        let pg_conn = &self.get()?;

        update(items.filter(id.eq(item_id)))
            .set(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Item, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<Item, Error>;

    fn handle(&mut self, FindById(item_id): FindById, _: &mut Self::Context) -> Self::Result {
        use schema::items::dsl::*;

        let pg_conn = &self.get()?;

        items
            .filter(id.eq(item_id))
            .first::<Item>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<usize, Error>")]
pub struct Delete(pub Uuid);

impl Handler<Delete> for PgExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, Delete(item_id): Delete, _: &mut Self::Context) -> Self::Result {
        use diesel::delete;
        use schema::items::dsl::*;

        let pg_conn = &self.get()?;

        delete(items.filter(id.eq(item_id)))
            .execute(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::postgres::{PgExecutor, PooledConnection};
use db::Error;
use models::payout::{Payout, PayoutPayload};
use types::{PayoutStatus, U128};

pub fn insert(payload: PayoutPayload, conn: &PooledConnection) -> Result<Payout, Error> {
    use diesel::insert_into;
    use schema::payouts::dsl;

    insert_into(dsl::payouts)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(id: Uuid, payload: PayoutPayload, conn: &PooledConnection) -> Result<Payout, Error> {
    use diesel::update;
    use schema::payouts::dsl;

    update(dsl::payouts.filter(dsl::id.eq(id)))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_all_confirmed(
    block_height: U128,
    conn: &PooledConnection,
) -> Result<Vec<Payout>, Error> {
    use schema::payouts::dsl;

    dsl::payouts
        .filter(
            dsl::status
                .eq(PayoutStatus::Pending)
                .and(dsl::eth_block_height_required.le(block_height)),
        )
        .load::<Payout>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct Insert(pub PayoutPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct Update(pub Uuid, pub PayoutPayload);

impl Handler<Update> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(&mut self, Update(id, payload): Update, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        update(id, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Payout>, Error>")]
pub struct FindAllConfirmed(pub U128);

impl Handler<FindAllConfirmed> for PgExecutor {
    type Result = Result<Vec<Payout>, Error>;

    fn handle(
        &mut self,
        FindAllConfirmed(block_height): FindAllConfirmed,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_all_confirmed(block_height, &conn)
    }
}

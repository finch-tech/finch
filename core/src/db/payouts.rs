use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::postgres::PgExecutor;
use db::Error;
use models::payout::{Payout, PayoutPayload};
use types::{PayoutStatus, U128};

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct Insert(pub PayoutPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::payouts::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(payouts)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
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
        use schema::payouts::dsl::*;

        let pg_conn = &self.get()?;

        payouts
            .filter(
                status
                    .eq(PayoutStatus::Pending)
                    .and(eth_block_height_required.le(block_height)),
            )
            .load::<Payout>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct UpdateById(pub Uuid, pub PayoutPayload);

impl Handler<UpdateById> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(
        &mut self,
        UpdateById(payout_id, payload): UpdateById,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::payouts::dsl::*;

        let pg_conn = &self.get()?;

        update(payouts.filter(id.eq(payout_id)))
            .set(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

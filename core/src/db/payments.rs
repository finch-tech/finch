use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::PgExecutor;
use db::Error;
use models::payment::{Payment, PaymentPayload};
use uuid::Uuid;

use types::{H160, PaymentStatus, U128};

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct Insert(pub PaymentPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        use diesel::insert_into;
        use schema::payments::dsl::*;

        let pg_conn = &self.get()?;

        insert_into(payments)
            .values(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(&mut self, FindById(payment_id): FindById, _: &mut Self::Context) -> Self::Result {
        use schema::payments::dsl::*;

        let pg_conn = &self.get()?;

        payments
            .filter(id.eq(payment_id))
            .first::<Payment>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Payment>, Error>")]
pub struct FindAllConfirmed(pub U128);

impl Handler<FindAllConfirmed> for PgExecutor {
    type Result = Result<Vec<Payment>, Error>;

    fn handle(
        &mut self,
        FindAllConfirmed(block_height): FindAllConfirmed,
        _: &mut Self::Context,
    ) -> Self::Result {
        use schema::payments::dsl::*;

        let pg_conn = &self.get()?;

        payments
            .filter(
                status
                    .eq(PaymentStatus::Paid)
                    .and(block_height_required.le(block_height)),
            )
            .load::<Payment>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Payment>, Error>")]
pub struct FindAllByEthAddress(pub Vec<H160>);

impl Handler<FindAllByEthAddress> for PgExecutor {
    type Result = Result<Vec<Payment>, Error>;

    fn handle(
        &mut self,
        FindAllByEthAddress(addresses): FindAllByEthAddress,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::pg::expression::dsl::any;
        use schema::payments::dsl::*;

        let pg_conn = &self.get()?;

        payments
            .filter(eth_address.eq(any(addresses)))
            .load::<Payment>(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct UpdateById(pub Uuid, pub PaymentPayload);

impl Handler<UpdateById> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(
        &mut self,
        UpdateById(payment_id, payload): UpdateById,
        _: &mut Self::Context,
    ) -> Self::Result {
        use diesel::update;
        use schema::payments::dsl::*;

        let pg_conn = &self.get()?;

        update(payments.filter(id.eq(payment_id)))
            .set(&payload)
            .get_result(pg_conn)
            .map_err(|e| Error::from(e))
    }
}

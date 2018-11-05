use actix::prelude::*;
use diesel::prelude::*;

use db::postgres::{PgExecutor, PooledConnection};
use db::Error;
use models::payment::{Payment, PaymentPayload};
use uuid::Uuid;

use types::H160;

pub fn insert(payload: PaymentPayload, conn: &PooledConnection) -> Result<Payment, Error> {
    use diesel::insert_into;
    use schema::payments::dsl;

    insert_into(dsl::payments)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(
    id: Uuid,
    payload: PaymentPayload,
    conn: &PooledConnection,
) -> Result<Payment, Error> {
    use diesel::update;
    use schema::payments::dsl;

    update(dsl::payments.filter(dsl::id.eq(id)))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_id(id: Uuid, conn: &PooledConnection) -> Result<Payment, Error> {
    use schema::payments::dsl;

    dsl::payments
        .filter(dsl::id.eq(id))
        .first::<Payment>(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_all_by_eth_addresses(
    addresses: Vec<H160>,
    conn: &PooledConnection,
) -> Result<Vec<Payment>, Error> {
    use diesel::pg::expression::dsl::any;
    use schema::payments::dsl;

    dsl::payments
        .filter(dsl::eth_address.eq(any(addresses)))
        .load::<Payment>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct Insert(pub PaymentPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct Update(pub Uuid, pub PaymentPayload);

impl Handler<Update> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(&mut self, Update(id, payload): Update, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        update(id, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payment, Error>")]
pub struct FindById(pub Uuid);

impl Handler<FindById> for PgExecutor {
    type Result = Result<Payment, Error>;

    fn handle(&mut self, FindById(id): FindById, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        find_by_id(id, &conn)
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
        let conn = &self.get()?;

        find_all_by_eth_addresses(addresses, &conn)
    }
}

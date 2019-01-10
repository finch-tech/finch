use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use db::{
    bitcoin::transactions as btc_transactions,
    ethereum::transactions as eth_transactions,
    payments,
    postgres::{PgExecutor, PooledConnection},
    Error,
};
use models::{
    bitcoin::Transaction as BtcTransaction,
    ethereum::Transaction as EthTransaction,
    payment::PaymentPayload,
    payout::{Payout, PayoutPayload},
};
use types::{currency::Crypto, PayoutStatus, U128};

pub fn insert_btc(
    payout_payload: PayoutPayload,
    payment_payload: PaymentPayload,
    transaction_payload: BtcTransaction,
    conn: &PooledConnection,
) -> Result<Payout, Error> {
    use diesel::insert_into;
    use schema::payouts::dsl;

    payments::update(payout_payload.payment_id.unwrap(), payment_payload, conn)?;

    btc_transactions::insert(transaction_payload, conn)?;

    insert_into(dsl::payouts)
        .values(&payout_payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn insert_eth(
    payout_payload: PayoutPayload,
    payment_payload: PaymentPayload,
    transaction_payload: EthTransaction,
    conn: &PooledConnection,
) -> Result<Payout, Error> {
    use diesel::insert_into;
    use schema::payouts::dsl;

    payments::update(payout_payload.payment_id.unwrap(), payment_payload, conn)?;

    eth_transactions::insert(transaction_payload, conn)?;

    insert_into(dsl::payouts)
        .values(&payout_payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

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

pub fn update_with_payment(
    id: Uuid,
    payout_payload: PayoutPayload,
    payment_payload: PaymentPayload,
    conn: &PooledConnection,
) -> Result<Payout, Error> {
    use diesel::update;
    use schema::payouts::dsl;

    payments::update(payout_payload.payment_id.unwrap(), payment_payload, conn)?;

    update(dsl::payouts.filter(dsl::id.eq(id)))
        .set(&payout_payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_all_confirmed(
    block_height: U128,
    typ: Crypto,
    conn: &PooledConnection,
) -> Result<Vec<Payout>, Error> {
    use schema::payouts::dsl;

    dsl::payouts
        .filter(
            dsl::status.eq(PayoutStatus::Pending).and(
                dsl::block_height_required
                    .le(block_height)
                    .and(dsl::typ.eq(typ)),
            ),
        )
        .load::<Payout>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct InsertBtc {
    pub payout_payload: PayoutPayload,
    pub payment_payload: PaymentPayload,
    pub transaction_payload: BtcTransaction,
}

impl Handler<InsertBtc> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(
        &mut self,
        InsertBtc {
            payout_payload,
            payment_payload,
            transaction_payload,
        }: InsertBtc,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        conn.transaction::<_, Error, _>(|| {
            insert_btc(payout_payload, payment_payload, transaction_payload, &conn)
        })
    }
}

#[derive(Message)]
#[rtype(result = "Result<Payout, Error>")]
pub struct InsertEth {
    pub payout_payload: PayoutPayload,
    pub payment_payload: PaymentPayload,
    pub transaction_payload: EthTransaction,
}

impl Handler<InsertEth> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(
        &mut self,
        InsertEth {
            payout_payload,
            payment_payload,
            transaction_payload,
        }: InsertEth,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        conn.transaction::<_, Error, _>(|| {
            insert_eth(payout_payload, payment_payload, transaction_payload, &conn)
        })
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
#[rtype(result = "Result<Payout, Error>")]
pub struct UpdateWithPayment {
    pub id: Uuid,
    pub payout_payload: PayoutPayload,
    pub payment_payload: PaymentPayload,
}

impl Handler<UpdateWithPayment> for PgExecutor {
    type Result = Result<Payout, Error>;

    fn handle(
        &mut self,
        UpdateWithPayment {
            id,
            payout_payload,
            payment_payload,
        }: UpdateWithPayment,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        conn.transaction::<_, Error, _>(|| {
            update_with_payment(id, payout_payload, payment_payload, &conn)
        })
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Payout>, Error>")]
pub struct FindAllConfirmed {
    pub block_height: U128,
    pub typ: Crypto,
}

impl Handler<FindAllConfirmed> for PgExecutor {
    type Result = Result<Vec<Payout>, Error>;

    fn handle(
        &mut self,
        FindAllConfirmed { block_height, typ }: FindAllConfirmed,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_all_confirmed(block_height, typ, &conn)
    }
}

use actix::prelude::*;
use diesel::prelude::*;

use db::{
    postgres::{PgExecutor, PooledConnection},
    Error,
};
use models::bitcoin::{BlockchainStatus, BlockchainStatusPayload};
use types::bitcoin::Network;

pub fn insert(
    payload: BlockchainStatusPayload,
    conn: &PooledConnection,
) -> Result<BlockchainStatus, Error> {
    use diesel::insert_into;
    use schema::btc_blockchain_statuses::dsl;

    insert_into(dsl::btc_blockchain_statuses)
        .values(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn update(
    network: Network,
    payload: BlockchainStatusPayload,
    conn: &PooledConnection,
) -> Result<BlockchainStatus, Error> {
    use diesel::update;
    use schema::btc_blockchain_statuses::dsl;

    update(dsl::btc_blockchain_statuses.filter(dsl::network.eq(network)))
        .set(&payload)
        .get_result(conn)
        .map_err(|e| Error::from(e))
}

pub fn find_by_network(
    network: Network,
    conn: &PooledConnection,
) -> Result<BlockchainStatus, Error> {
    use schema::btc_blockchain_statuses::dsl;

    dsl::btc_blockchain_statuses
        .filter(dsl::network.eq(network))
        .first::<BlockchainStatus>(conn)
        .map_err(|e| Error::from(e))
}

#[derive(Message)]
#[rtype(result = "Result<BlockchainStatus, Error>")]
pub struct Insert(pub BlockchainStatusPayload);

impl Handler<Insert> for PgExecutor {
    type Result = Result<BlockchainStatus, Error>;

    fn handle(&mut self, Insert(payload): Insert, _: &mut Self::Context) -> Self::Result {
        let conn = &self.get()?;

        insert(payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<BlockchainStatus, Error>")]
pub struct Update {
    pub network: Network,
    pub payload: BlockchainStatusPayload,
}
impl Handler<Update> for PgExecutor {
    type Result = Result<BlockchainStatus, Error>;

    fn handle(
        &mut self,
        Update { network, payload }: Update,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        update(network, payload, &conn)
    }
}

#[derive(Message)]
#[rtype(result = "Result<BlockchainStatus, Error>")]
pub struct FindByNetwork(pub Network);

impl Handler<FindByNetwork> for PgExecutor {
    type Result = Result<BlockchainStatus, Error>;

    fn handle(
        &mut self,
        FindByNetwork(network): FindByNetwork,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = &self.get()?;

        find_by_network(network, &conn)
    }
}

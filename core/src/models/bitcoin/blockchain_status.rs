use futures::Future;

use db::{
    bitcoin::blockchain_statuses::{FindByNetwork, Insert, Update},
    postgres::PgExecutorAddr,
};
use models::Error;
use schema::btc_blockchain_statuses;
use types::{bitcoin::Network, U128};

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name = "btc_blockchain_statuses"]
pub struct BlockchainStatusPayload {
    pub network: Option<Network>,
    pub block_height: Option<U128>,
}

#[derive(Queryable, Serialize)]
pub struct BlockchainStatus {
    pub network: Network,
    pub block_height: U128,
}

impl BlockchainStatus {
    pub fn insert(
        payload: BlockchainStatusPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = BlockchainStatus, Error = Error> {
        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find(
        network: Network,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = BlockchainStatus, Error = Error> {
        (*postgres)
            .send(FindByNetwork(network))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        network: Network,
        payload: BlockchainStatusPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = BlockchainStatus, Error = Error> {
        (*postgres)
            .send(Update { network, payload })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }
}

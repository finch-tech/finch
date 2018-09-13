use base64::encode;
use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::client_tokens::{Delete, FindById, FindByToken, Insert};
use db::postgres::PgExecutorAddr;
use models::store::Store;
use models::Error;
use schema::client_tokens;
use types::Client;

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[table_name = "client_tokens"]
pub struct ClientTokenPayload {
    pub id: Option<Uuid>,
    pub name: String,
    pub token: Option<Uuid>,
    pub store_id: Uuid,
    // TODO: Use more strict type.
    pub referer: String,
    pub created_at: Option<DateTime<Utc>>,
    pub typ: Client,
}

impl ClientTokenPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }
}

#[derive(Debug, Identifiable, Queryable, Serialize, Associations)]
#[belongs_to(Store, foreign_key = "store_id")]
pub struct ClientToken {
    pub id: Uuid,
    pub name: String,
    pub token: Uuid,
    pub store_id: Uuid,
    // TODO: Use more strict type.
    pub referer: String,
    pub created_at: DateTime<Utc>,
    pub typ: Client,
}

impl ClientToken {
    pub fn insert(
        mut payload: ClientTokenPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = ClientToken, Error = Error> {
        payload.set_created_at();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = ClientToken, Error = Error> {
        (*postgres)
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_token(
        token: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = ClientToken, Error = Error> {
        (*postgres)
            .send(FindByToken(token))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn delete(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = usize, Error = Error> {
        (*postgres)
            .send(Delete(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        let token = encode(self.token.as_bytes());
        json!({
            "id": self.id,
            "name": self.name,
            "token": token,
            "store_id": self.store_id,
            "referer": self.referer,
            "created_at": self.created_at.timestamp(),
            "typ": self.typ,
        })
    }
}

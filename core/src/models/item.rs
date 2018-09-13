use bigdecimal::BigDecimal;
use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::items::{FindById, Insert, Update};
use db::postgres::PgExecutorAddr;
use models::store::Store;
use models::Error;
use schema::items;
use types::U128;

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[table_name = "items"]
pub struct ItemPayload {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub store_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub price: Option<BigDecimal>,
    pub confirmations_required: Option<U128>,
}

impl ItemPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }
}

#[derive(Identifiable, Queryable, Associations, Serialize)]
#[belongs_to(Store, foreign_key = "store_id")]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub store_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub price: BigDecimal,
    pub confirmations_required: U128,
}

impl Item {
    pub fn insert(
        mut payload: ItemPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Item, Error = Error> {
        payload.set_created_at();
        payload.set_updated_at();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        item_id: Uuid,
        mut payload: ItemPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Item, Error = Error> {
        payload.set_updated_at();

        (*postgres)
            .send(Update { item_id, payload })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = Item, Error = Error> {
        (*postgres)
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "price": format!("{}", self.price),
        })
    }
}

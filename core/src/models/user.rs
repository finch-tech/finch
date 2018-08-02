use chrono::prelude::*;
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::postgres::PgExecutorAddr;
use db::users::{FindByEmail, FindById, Insert};
use models::Error;
use schema::users;

#[derive(Insertable, AsChangeset, Deserialize)]
#[table_name = "users"]
pub struct UserPayload {
    pub email: String,
    pub password: String,
    pub salt: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub active: bool,
}

impl UserPayload {
    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }
}

#[derive(Identifiable, Queryable, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

impl User {
    pub fn insert(
        mut payload: UserPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        payload.set_created_at();
        payload.set_updated_at();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_email(
        email: String,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        (*postgres)
            .send(FindByEmail(email))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_id(
        id: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        (*postgres)
            .send(FindById(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn export(&self) -> Value {
        json!({
            "id": self.id,
            "email": self.email,
            "created_at": self.created_at.timestamp(),
            "updated_at": self.updated_at.timestamp(),
        })
    }
}

use std::convert::From;

use chrono::{prelude::*, Duration};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

use db::postgres::PgExecutorAddr;
use db::users::{
    Activate, Delete, DeleteExpired, FindByEmail, FindById, FindByResetToken, Insert, Update,
};
use models::Error;
use schema::users;

#[derive(Insertable, AsChangeset, Deserialize, Clone)]
#[table_name = "users"]
pub struct UserPayload {
    pub email: Option<String>,
    pub password: Option<String>,
    pub salt: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_verified: Option<bool>,
    pub verification_token: Option<Uuid>,
    pub verification_token_expires_at: Option<DateTime<Utc>>,
    pub reset_token: Option<Option<Uuid>>,
    pub reset_token_expires_at: Option<Option<DateTime<Utc>>>,
}

impl UserPayload {
    pub fn new() -> Self {
        UserPayload {
            email: None,
            password: None,
            salt: None,
            created_at: None,
            updated_at: None,
            is_verified: None,
            verification_token: None,
            verification_token_expires_at: None,
            reset_token: None,
            reset_token_expires_at: None,
        }
    }

    pub fn set_created_at(&mut self) {
        self.created_at = Some(Utc::now());
    }

    pub fn set_updated_at(&mut self) {
        self.updated_at = Some(Utc::now());
    }

    pub fn set_verification_token(&mut self) {
        self.is_verified = Some(false);
        self.verification_token = Some(Uuid::new_v4());
        self.verification_token_expires_at = Some(Utc::now() + Duration::days(1));
    }

    pub fn set_reset_token(&mut self) {
        self.reset_token = Some(Some(Uuid::new_v4()));
        self.reset_token_expires_at = Some(Some(Utc::now() + Duration::days(1)));
    }
}

impl From<User> for UserPayload {
    fn from(user: User) -> Self {
        UserPayload {
            email: Some(user.email),
            password: Some(user.password),
            salt: Some(user.salt),
            created_at: Some(user.created_at),
            updated_at: Some(user.updated_at),
            is_verified: Some(user.is_verified),
            verification_token: Some(user.verification_token),
            verification_token_expires_at: Some(user.verification_token_expires_at),
            reset_token: Some(user.reset_token),
            reset_token_expires_at: Some(user.reset_token_expires_at),
        }
    }
}

#[derive(Identifiable, Queryable, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_verified: bool,
    pub verification_token: Uuid,
    pub verification_token_expires_at: DateTime<Utc>,
    pub reset_token: Option<Uuid>,
    pub reset_token_expires_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn insert(
        mut payload: UserPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        payload.set_created_at();
        payload.set_updated_at();
        payload.set_verification_token();

        (*postgres)
            .send(Insert(payload))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn update(
        id: Uuid,
        mut payload: UserPayload,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        payload.set_updated_at();

        (*postgres)
            .send(Update { id, payload })
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn find_by_reset_token(
        token: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        (*postgres)
            .send(FindByResetToken(token))
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

    pub fn activate(
        token: Uuid,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = User, Error = Error> {
        (*postgres)
            .send(Activate(token))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn delete(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = usize, Error = Error> {
        (*postgres)
            .send(Delete(id))
            .from_err()
            .and_then(|res| res.map_err(|e| Error::from(e)))
    }

    pub fn delete_expired(
        email: String,
        postgres: &PgExecutorAddr,
    ) -> impl Future<Item = usize, Error = Error> {
        (*postgres)
            .send(DeleteExpired(email))
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

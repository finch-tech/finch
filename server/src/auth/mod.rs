use actix_web::{error, Error as ActixError, FromRequest, HttpMessage, HttpRequest};
use base64::decode;
use chrono::prelude::*;
use futures::future::{err, Future};
use jwt;
use uuid::Uuid;

use models::client_token::ClientToken;
use server::AppState;
use services;
use types::PrivateKey;

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTPayload {
    pub client: Option<AuthClient>,
    pub user: Option<AuthUser>,
}

impl JWTPayload {
    pub fn new(user: Option<AuthUser>, client: Option<AuthClient>) -> Self {
        JWTPayload { client, user }
    }

    pub fn encode(&self, jwt_private: &PrivateKey) -> Result<String, jwt::errors::Error> {
        let mut header = jwt::Header::default();
        header.alg = jwt::Algorithm::RS256;

        jwt::encode(&header, &self, jwt_private)
    }
}

impl FromRequest<AppState> for JWTPayload {
    type Config = ();
    type Result = Result<JWTPayload, ActixError>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        let state = req.state();

        let auth_header = match req.headers().get("authorization") {
            Some(auth_header) => auth_header,
            None => return Err(error::ErrorUnauthorized("Invalid authorization token.")),
        };

        let auth_header_parts: Vec<_> = auth_header.to_str().unwrap().split_whitespace().collect();
        if auth_header_parts.len() != 2 {
            return Err(error::ErrorUnauthorized("Invalid authorization token."));
        }

        if auth_header_parts.len() != 2 || auth_header_parts[0].to_lowercase() != "bearer" {
            return Err(error::ErrorUnauthorized("Invalid authorization token."));
        }

        let validation = jwt::Validation {
            algorithms: vec![jwt::Algorithm::RS256],
            ..Default::default()
        };

        match jwt::decode::<JWTPayload>(&auth_header_parts[1], &state.jwt_public, &validation) {
            Ok(token) => Ok(token.claims),
            Err(_) => Err(error::ErrorUnauthorized("Invalid authorization token.")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClient {
    pub id: Uuid,
    pub store_id: Uuid,
    pub created_at: i64,
}

impl AuthClient {
    pub fn new(client_token: ClientToken) -> Self {
        AuthClient {
            id: Uuid::new_v4(),
            store_id: client_token.store_id,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl FromRequest<AppState> for AuthClient {
    type Config = ();
    type Result = Result<AuthClient, ActixError>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        let token = JWTPayload::extract(&req)?;

        match token.client {
            Some(ref client) => Ok((*client).clone()),
            None => Err(error::ErrorUnauthorized("Invalid authorization token.")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequest<AppState> for AuthUser {
    type Config = ();
    type Result = Result<AuthUser, ActixError>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        let token = JWTPayload::extract(&req)?;

        match token.user {
            Some(ref user) => Ok((*user).clone()),
            None => Err(error::ErrorUnauthorized("Invalid authorization token.")),
        }
    }
}

impl FromRequest<AppState> for ClientToken {
    type Config = ();
    type Result = Box<Future<Item = ClientToken, Error = ActixError>>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        let state = req.state();

        let auth_header = match req.headers().get("authorization") {
            Some(auth_header) => auth_header,
            None => {
                return Box::new(err(error::ErrorUnauthorized(
                    "Invalid authorization token.",
                )))
            }
        };

        let auth_header_parts: Vec<_> = auth_header.to_str().unwrap().split_whitespace().collect();

        if auth_header_parts.len() != 2 {
            return Box::new(err(error::ErrorUnauthorized(
                "Invalid authorization token.",
            )));
        }

        if auth_header_parts.len() != 2 || auth_header_parts[0].to_lowercase() != "bearer" {
            return Box::new(err(error::ErrorUnauthorized(
                "Invalid authorization token.",
            )));
        }

        let token = match decode(&auth_header_parts[1]) {
            Ok(decoded) => match Uuid::from_bytes(&decoded) {
                Ok(token) => token,
                Err(_) => {
                    return Box::new(err(error::ErrorUnauthorized(
                        "Invalid authorization token.",
                    )))
                }
            },
            Err(_) => {
                return Box::new(err(error::ErrorUnauthorized(
                    "Invalid authorization token.",
                )))
            }
        };

        // TODO: Check referer.
        Box::new(services::client_tokens::get_by_token(token, state.postgres.clone()).from_err())
    }
}

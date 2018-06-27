use actix_web::{error, Error as ActixError, FromRequest};
use actix_web::{HttpMessage, HttpRequest};
use jwt;
use uuid;

use server::AppState;
use types::PrivateKey;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

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
    pub name: String,
}

fn is_valid_client(client: &AuthClient) -> bool {
    // TODO: Client authentication.
    true
}

impl FromRequest<AppState> for AuthClient {
    type Config = ();
    type Result = Result<AuthClient, ActixError>;

    fn from_request(req: &HttpRequest<AppState>, _cfg: &Self::Config) -> Self::Result {
        let token = JWTPayload::extract(&req)?;

        match token.client {
            Some(ref client) => {
                if !is_valid_client(&client) {
                    return Err(error::ErrorUnauthorized("Invalid authorization token."));
                }

                Ok((*client).clone())
            }
            None => Err(error::ErrorUnauthorized("Invalid authorization token.")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub id: uuid::Uuid,
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

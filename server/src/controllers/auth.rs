use actix_web::{Json, State};
use futures::future::Future;
use serde_json::Value;

use auth::AuthUser;
use models::user::UserPayload;
use server::AppState;
use services::users::LoginParams;
use services::{self, Error};

#[derive(Debug, Deserialize)]
pub struct RegistrationParams {
    pub email: String,
    pub password: String,
}

pub fn registration(
    (state, params): (State<AppState>, Json<RegistrationParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    let params = params.into_inner();

    let payload = UserPayload {
        email: params.email,
        password: params.password,
        salt: None,
        created_at: None,
        updated_at: None,
        active: true,
    };

    services::users::register(payload, state.postgres)
        .then(|res| res.and_then(|user| Ok(Json(user.export()))))
}

pub fn authentication(
    (state, params): (State<AppState>, Json<LoginParams>),
) -> impl Future<Item = String, Error = Error> {
    let state = state.clone();
    services::users::authenticate(params.into_inner(), state.postgres, state.jwt_private)
}

pub fn profile(
    (state, user): (State<AppState>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    services::users::get(user.id, state.postgres)
        .then(|res| res.and_then(|user| Ok(Json(user.export()))))
}

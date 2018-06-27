use actix_web::{Json, State};
use futures::future::Future;
use serde_json::Value;

use auth::{AuthUser, LoginPayload};
use models::user::UserPayload;
use server::AppState;
use services::{self, Error};

pub fn registration(
    (state, payload): (State<AppState>, Json<UserPayload>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    services::users::register(payload.into_inner(), state.postgres)
        .then(|res| res.and_then(|user| Ok(Json(user.export()))))
}

pub fn authentication(
    (state, payload): (State<AppState>, Json<LoginPayload>),
) -> impl Future<Item = String, Error = Error> {
    let state = state.clone();
    services::users::authenticate(payload.into_inner(), state.postgres, state.jwt_private)
}

pub fn profile(
    (state, user): (State<AppState>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    services::users::get(user.id, state.postgres)
        .then(|res| res.and_then(|user| Ok(Json(user.export()))))
}

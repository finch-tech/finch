use actix_web::{Json, State};
use futures::future::{err, Future};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::user::UserPayload;
use server::AppState;
use services::{self, Error};

#[derive(Debug, Deserialize)]
pub struct RegistrationParams {
    pub email: String,
    pub password: String,
}

pub fn registration(
    (state, params): (State<AppState>, Json<RegistrationParams>),
) -> Box<Future<Item = Json<Value>, Error = Error>> {
    let params = params.into_inner();

    if params.email.len() == 0 || params.password.len() == 0 {
        return Box::new(err(Error::BadRequest));
    }

    let payload = UserPayload {
        email: Some(params.email),
        password: Some(params.password),
        salt: None,
        created_at: None,
        updated_at: None,
        is_verified: None,
        verification_token: None,
        verification_token_expires_at: None,
        reset_token: None,
        reset_token_expires_at: None,
        active: Some(true),
    };

    Box::new(
        services::users::register(payload, state.mailer.clone(), &state.postgres)
            .then(|res| res.and_then(|user| Ok(Json(user.export())))),
    )
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

pub fn authentication(
    (state, params): (State<AppState>, Json<LoginParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::users::authenticate(
        params.email,
        params.password,
        &state.postgres,
        state.jwt_private.clone(),
    ).then(|res| {
        res.and_then(|(token, user)| Ok(Json(json!({ "token": token, "user": user.export() }))))
    })
}

#[derive(Deserialize)]
pub struct ActivationParams {
    pub token: Uuid,
}

pub fn activation(
    (state, params): (State<AppState>, Json<ActivationParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::users::activate(params.token, &state.postgres, state.jwt_private.clone()).then(
        |res| {
            res.and_then(|(token, user)| Ok(Json(json!({ "token": token, "user": user.export() }))))
        },
    )
}

#[derive(Deserialize)]
pub struct ResetPasswordParams {
    pub email: String,
}

pub fn reset_password(
    (state, params): (State<AppState>, Json<ResetPasswordParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::users::reset_password(params.email, state.mailer.clone(), &state.postgres)
        .then(|res| res.and_then(|_| Ok(Json(json!({})))))
}

#[derive(Deserialize)]
pub struct ChangePasswordParams {
    pub token: Uuid,
    pub password: String,
}

pub fn change_password(
    (state, params): (State<AppState>, Json<ChangePasswordParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::users::change_password(
        params.token,
        params.password,
        &state.postgres,
        state.jwt_private.clone(),
    ).then(|res| {
        res.and_then(|(token, user)| Ok(Json(json!({ "token": token, "user": user.export() }))))
    })
}

pub fn profile(
    (state, user): (State<AppState>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    services::users::get(user.id, &state.postgres)
        .then(|res| res.and_then(|user| Ok(Json(user.export()))))
}

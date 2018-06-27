use actix_web::{Json, Path, State};
use futures::future::Future;
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use models::store::StorePayload;
use server::AppState;
use services::{self, Error};

pub fn create(
    (state, mut payload, user): (State<AppState>, Json<StorePayload>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();

    payload.owner_id = Some(user.id);
    services::stores::create(payload.into_inner(), state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

// TODO: Client auth
pub fn get(
    (state, path): (State<AppState>, Path<Uuid>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    let id = path.into_inner();

    services::stores::get(id, state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

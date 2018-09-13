use actix_web::{Json, Path, Query, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::client_token::ClientTokenPayload;
use core::store::Store;
use server::AppState;
use services::{self, Error};
use types::Client;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub referer: String,
    pub typ: Client,
    pub store_id: Uuid,
}

fn validate_store_owner(store: &Store, user: &AuthUser) -> Result<bool, Error> {
    if store.owner_id != user.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn create(
    (state, user, params): (State<AppState>, AuthUser, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::stores::get(params.store_id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                let payload = ClientTokenPayload {
                    id: None,
                    name: params.name,
                    token: None,
                    store_id: store.id,
                    referer: params.referer,
                    created_at: None,
                    typ: params.typ,
                };

                services::client_tokens::create(payload, &state.postgres)
                    .then(|res| res.and_then(|client_token| Ok(Json(client_token.export()))))
            })
    })
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub store_id: Uuid,
}

pub fn list(
    (state, params, user): (State<AppState>, Query<ListParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    services::stores::get(params.store_id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                services::client_tokens::find_by_store(store.id, &state.postgres).then(|res| {
                    res.and_then(|client_tokens| {
                        let mut exported = Vec::new();
                        client_tokens
                            .into_iter()
                            .for_each(|client_token| exported.push(client_token.export()));
                        Ok(Json(json!(exported)))
                    })
                })
            })
    })
}

pub fn delete(
    (state, path, user): (State<AppState>, Path<Uuid>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();

    services::client_tokens::get(id, &state.postgres).and_then(move |client_token| {
        services::stores::get(client_token.store_id, &state.postgres).and_then(move |store| {
            validate_store_owner(&store, &user)
                .into_future()
                .and_then(move |_| {
                    services::client_tokens::delete(id, &state.postgres)
                        .then(|res| res.and_then(|res| Ok(Json(json!({ "deleted": res })))))
                })
        })
    })
}

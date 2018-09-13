use actix_web::{Json, Path, Query, State};
use bigdecimal::BigDecimal;
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::item::ItemPayload;
use core::store::Store;
use server::AppState;
use services::{self, Error};
use types::U128;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub description: String,
    pub store_id: Uuid,
    pub price: BigDecimal,
    pub confirmations_required: U128,
}

fn validate_store_owner(store: &Store, user: &AuthUser) -> Result<bool, Error> {
    if store.owner_id != user.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn create(
    (state, params, user): (State<AppState>, Json<CreateParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::stores::get(params.store_id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                let payload = ItemPayload {
                    id: None,
                    name: Some(params.name),
                    description: Some(params.description),
                    store_id: Some(store.id),
                    created_at: None,
                    updated_at: None,
                    price: Some(params.price),
                    confirmations_required: Some(params.confirmations_required),
                };

                services::items::create(payload, &state.postgres)
                    .then(|res| res.and_then(|item| Ok(Json(item.export()))))
            })
    })
}

#[derive(Debug, Deserialize)]
pub struct PatchParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<BigDecimal>,
    pub confirmations_required: Option<U128>,
}

pub fn patch(
    (state, path, params, user): (State<AppState>, Path<Uuid>, Json<PatchParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();
    let params = params.into_inner();

    services::items::get(id, &state.postgres).and_then(move |item| {
        services::stores::get(item.store_id, &state.postgres).and_then(move |store| {
            validate_store_owner(&store, &user)
                .into_future()
                .and_then(move |_| {
                    let payload = ItemPayload {
                        id: None,
                        name: params.name,
                        description: params.description,
                        store_id: None,
                        created_at: None,
                        updated_at: None,
                        price: params.price,
                        confirmations_required: params.confirmations_required,
                    };

                    services::items::patch(id, payload, &state.postgres)
                        .then(|res| res.and_then(|item| Ok(Json(item.export()))))
                })
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
                services::items::find_by_store(store.id, &state.postgres).then(|res| {
                    res.and_then(|items| {
                        let mut exported = Vec::new();
                        items
                            .into_iter()
                            .for_each(|item| exported.push(item.export()));
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

    services::items::get(id, &state.postgres).and_then(move |item| {
        services::stores::get(item.store_id, &state.postgres).and_then(move |store| {
            validate_store_owner(&store, &user)
                .into_future()
                .and_then(move |_| {
                    services::items::delete(id, &state.postgres)
                        .then(|res| res.and_then(|res| Ok(Json(json!({ "deleted": res })))))
                })
        })
    })
}

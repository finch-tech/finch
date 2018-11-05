use actix_web::{Json, Path, Query, State};
use futures::future::{err, Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::store::{Store, StorePayload};
use server::AppState;
use services::{self, Error};
use types::{Currency, H160, U128};

const LIMIT: i64 = 15;
const OFFSET: i64 = 0;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub description: String,
}

pub fn create(
    (state, params, user): (State<AppState>, Json<CreateParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let mut params = params.into_inner();

    if params.name.len() == 0 {
        params.name = String::from("My Store");
    }

    let payload = StorePayload {
        id: None,
        name: Some(params.name),
        description: Some(params.description),
        owner_id: Some(user.id),
        private_key: None,
        public_key: None,
        created_at: None,
        updated_at: None,
        eth_payout_addresses: None,
        eth_confirmations_required: None,
        mnemonic: None,
        hd_path: None,
        base_currency: Some(Currency::Usd),
        deleted_at: None,
    };

    services::stores::create(payload, &state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

#[derive(Debug, Deserialize)]
pub struct PatchParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub eth_payout_addresses: Option<Vec<H160>>,
    pub eth_confirmations_required: Option<U128>,
}

fn validate_store_owner(store: &Store, user: &AuthUser) -> Result<bool, Error> {
    if store.owner_id != user.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn patch(
    (state, path, params, user): (State<AppState>, Path<Uuid>, Json<PatchParams>, AuthUser),
) -> Box<Future<Item = Json<Value>, Error = Error>> {
    let id = path.into_inner();
    let mut params = params.into_inner();

    if params.name.is_some() && params.name.clone().unwrap().len() == 0 {
        params.name = Some(String::from("My Store"));
    }

    if params.eth_confirmations_required.is_some()
        && params.eth_confirmations_required.unwrap() < U128::from(1)
    {
        return Box::new(err(Error::BadRequest));
    }

    Box::new(
        services::stores::get(id, &state.postgres).and_then(move |store| {
            validate_store_owner(&store, &user)
                .into_future()
                .and_then(move |_| {
                    let mut payload = StorePayload::new();

                    if let Some(name) = params.name {
                        payload.name = Some(name);
                    }

                    if let Some(description) = params.description {
                        payload.description = Some(description);
                    }

                    if let Some(eth_payout_addresses) = params.eth_payout_addresses {
                        payload.eth_payout_addresses = Some(Some(eth_payout_addresses));
                    }

                    if let Some(eth_confirmations_required) = params.eth_confirmations_required {
                        payload.eth_confirmations_required = Some(Some(eth_confirmations_required));
                    }

                    services::stores::patch(id, payload, &state.postgres)
                        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
                })
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn list(
    (state, params, user): (State<AppState>, Query<ListParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let mut limit = LIMIT;
    let mut offset = OFFSET;

    if let Some(_limit) = params.limit {
        if _limit < LIMIT {
            limit = _limit;
        }
    };

    if let Some(_offset) = params.offset {
        offset = _offset;
    };

    services::stores::find_by_owner(user.id, limit, offset, &state.postgres).then(move |res| {
        res.and_then(|stores| {
            let mut exported = Vec::new();
            stores
                .into_iter()
                .for_each(|store| exported.push(store.export()));
            Ok(Json(json!({
                "stores": exported,
                "limit": limit,
                "offset": offset,
            })))
        })
    })
}

pub fn get(
    (state, path, user): (State<AppState>, Path<Uuid>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();

    services::stores::get(id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                services::stores::get(id, &state.postgres)
                    .then(|res| res.and_then(|store| Ok(Json(store.export()))))
            })
    })
}

pub fn delete(
    (state, path, user): (State<AppState>, Path<Uuid>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();

    services::stores::get(id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                services::stores::delete(id, &state.postgres)
                    .then(|res| res.and_then(|res| Ok(Json(json!({ "deleted": res })))))
            })
    })
}

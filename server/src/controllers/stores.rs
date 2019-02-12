use actix_web::{Json, Path, Query, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::store::{Store, StorePayload};
use services::{self, Error};
use state::AppState;
use types::{bitcoin::Address as BtcAddress, H160};

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

    let mut payload = StorePayload::new();
    payload.name = Some(params.name);
    payload.description = Some(params.description);
    payload.owner_id = Some(user.id);
    payload.eth_confirmations_required = Some(Some(1));
    payload.btc_confirmations_required = Some(Some(1));

    services::stores::create(payload, state.btc_network, &state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

#[derive(Debug, Deserialize)]
pub struct PatchParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub eth_payout_addresses: Option<Vec<H160>>,
    pub eth_confirmations_required: Option<i32>,
    pub btc_payout_addresses: Option<Vec<BtcAddress>>,
    pub btc_confirmations_required: Option<i32>,
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

                    if let Some(btc_payout_address) = params.btc_payout_addresses {
                        payload.btc_payout_addresses = Some(Some(btc_payout_address));
                    }

                    if let Some(btc_confirmations_required) = params.btc_confirmations_required {
                        payload.btc_confirmations_required = Some(Some(btc_confirmations_required));
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
            let exported: Vec<Value> = stores.into_iter().map(|store| store.export()).collect();

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

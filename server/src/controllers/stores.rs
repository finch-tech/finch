use actix_web::{Json, Path, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::store::{Store, StorePayload};
use currency_api_client::Api as CurrencyApi;
use server::AppState;
use services::{self, Error};
use types::{Currency, H160};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub description: String,
    pub payout_addresses: Vec<H160>,
    pub base_currency: Currency,
    pub currency_api: CurrencyApi,
    pub currency_api_key: String,
}

pub fn create(
    (state, params, user): (State<AppState>, Json<CreateParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    // TODO: Check if the currency is legal tender.

    let payload = StorePayload {
        id: None,
        name: Some(params.name),
        description: Some(params.description),
        owner_id: user.id,
        private_key: None,
        public_key: None,
        created_at: None,
        updated_at: None,
        payout_addresses: Some(params.payout_addresses),
        mnemonic: None,
        hd_path: None,
        base_currency: Some(params.base_currency),
        currency_api: Some(params.currency_api),
        currency_api_key: Some(params.currency_api_key),
        active: true,
    };

    services::stores::create(payload, &state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

#[derive(Debug, Deserialize)]
pub struct PatchParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub payout_addresses: Option<Vec<H160>>,
    pub base_currency: Option<Currency>,
    pub currency_api: Option<CurrencyApi>,
    pub currency_api_key: Option<String>,
}

fn validate_store_owner(store: &Store, user: &AuthUser) -> Result<bool, Error> {
    if store.owner_id != user.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn patch(
    (state, path, params, user): (State<AppState>, Path<Uuid>, Json<PatchParams>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();
    let params = params.into_inner();

    services::stores::get(id, &state.postgres).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                let payload = StorePayload {
                    id: None,
                    name: params.name,
                    description: params.description,
                    owner_id: user.id,
                    private_key: None,
                    public_key: None,
                    created_at: None,
                    updated_at: None,
                    payout_addresses: params.payout_addresses,
                    mnemonic: None,
                    hd_path: None,
                    base_currency: params.base_currency,
                    currency_api: params.currency_api,
                    currency_api_key: params.currency_api_key,
                    active: true,
                };

                services::stores::patch(id, payload, &state.postgres)
                    .then(|res| res.and_then(|store| Ok(Json(store.export()))))
            })
    })
}

pub fn list(
    (state, user): (State<AppState>, AuthUser),
) -> impl Future<Item = Json<Value>, Error = Error> {
    services::stores::find_by_owner(user.id, &state.postgres).then(|res| {
        res.and_then(|stores| {
            let mut exported = Vec::new();
            stores
                .into_iter()
                .for_each(|store| exported.push(store.export()));
            Ok(Json(json!(exported)))
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

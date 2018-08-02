use actix_web::{Json, Path, State};
use futures::future::Future;
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::store::StorePayload;
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

    services::stores::create(payload, &state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

// TODO: Client auth
pub fn get(
    (state, path): (State<AppState>, Path<Uuid>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();

    services::stores::get(id, &state.postgres)
        .then(|res| res.and_then(|store| Ok(Json(store.export()))))
}

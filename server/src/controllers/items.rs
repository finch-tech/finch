use actix_web::{Json, State};
use bigdecimal::BigDecimal;
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthUser;
use core::item::ItemPayload;
use core::store::Store;
use server::AppState;
use services::{self, Error};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub description: Option<String>,
    pub store_id: Uuid,
    pub price: BigDecimal,
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
    let state = state.clone();
    let params = params.into_inner();

    services::stores::get(params.store_id, state.postgres.clone()).and_then(move |store| {
        validate_store_owner(&store, &user)
            .into_future()
            .and_then(move |_| {
                let payload = ItemPayload {
                    id: None,
                    name: params.name,
                    description: params.description,
                    store_id: store.id,
                    created_at: None,
                    updated_at: None,
                    price: params.price,
                };

                services::items::create(payload, state.postgres.clone())
                    .then(|res| res.and_then(|item| Ok(Json(item.export()))))
            })
    })
}

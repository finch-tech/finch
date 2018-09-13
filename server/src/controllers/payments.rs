use std::collections::HashSet;

use actix_web::{Json, Path, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::app_status::AppStatus;
use core::client_token::ClientToken;
use core::item::Item;
use core::payment::{Payment, PaymentPayload};
use server::AppState;
use services::{self, Error};
use types::Currency;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub currencies: HashSet<Currency>,
    pub item_id: Uuid,
}

fn validate_item(item: &Item, client: &AuthClient) -> Result<bool, Error> {
    if item.store_id != client.store_id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn create(
    (state, client_token, params): (State<AppState>, ClientToken, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();
    // TODO: Check params.currencies length.

    services::items::get(params.item_id, &state.postgres).and_then(move |item| {
        let auth_client = AuthClient::new(client_token);

        validate_item(&item, &auth_client)
            .into_future()
            .and_then(move |_| {
                let payload = PaymentPayload {
                    id: None,
                    status: None,
                    store_id: auth_client.store_id,
                    item_id: item.id,
                    created_by: auth_client.id,
                    created_at: None,
                    expires_at: None,
                    paid_at: None,
                    index: None,
                    eth_address: None,
                    eth_price: None,
                    btc_address: None,
                    btc_price: None,
                    confirmations_required: item.confirmations_required,
                    block_height_required: None,
                    transaction_hash: None,
                    payout_transaction_hash: None,
                };

                services::payments::create(params.currencies, payload, &state.postgres).and_then(
                    move |payment| {
                        payment
                            .item(&state.postgres)
                            .from_err()
                            .and_then(move |item| {
                                JWTPayload::new(None, Some(auth_client), payment.expires_at)
                                    .encode(&state.jwt_private)
                                    .map_err(|e| Error::from(e))
                                    .into_future()
                                    .then(move |res| {
                                        res.and_then(|auth_token| {
                                            Ok(Json(json!({
                                            "payment": payment.export(),
                                            "item": item.export(),
                                            "token": auth_token
                                        })))
                                        })
                                    })
                            })
                    },
                )
            })
    })
}

fn validate_client(payment: &Payment, client: &AuthClient) -> Result<bool, Error> {
    if payment.created_by != client.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn get_status(
    (state, client, path): (State<AppState>, AuthClient, Path<Uuid>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();

    services::payments::get(id, &state.postgres).and_then(move |payment| {
        validate_client(&payment, &client)
            .into_future()
            .and_then(move |_| {
                AppStatus::find(&state.postgres)
                    .from_err()
                    .and_then(move |status| match payment.block_height_required {
                        Some(block_height_required) => Ok(Json(json!({
                            "status": payment.status,
                            "confirmations_required": format!("{}", payment.confirmations_required),
                            "block_height": format!("{}", status.block_height.unwrap()),
                            "block_height_required": format!("{}", block_height_required)
                        }))),
                        None => Ok(Json(json!({
                            "status": payment.status,
                            "confirmations_required": format!("{}", payment.confirmations_required),
                            "block_height": format!("{}", status.block_height.unwrap()),
                        }))),
                    })
            })
    })
}

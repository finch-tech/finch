use std::collections::HashSet;

use actix_web::{Json, Path, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::client_token::ClientToken;
use core::payment::{Payment, PaymentPayload};
use server::AppState;
use services::{self, Error};
use types::Currency;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub currencies: HashSet<Currency>,
    pub item_id: Uuid,
}

pub fn create(
    (state, client_token, params): (State<AppState>, ClientToken, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let state = state.clone();
    let params = params.into_inner();
    // TODO: Check params.currencies length.

    let auth_client = AuthClient::new(client_token);

    let payload = PaymentPayload {
        id: None,
        status: None,
        store_id: auth_client.store_id,
        item_id: params.item_id,
        created_by: auth_client.id,
        created_at: None,
        paid_at: None,
        index: None,
        eth_address: None,
        eth_price: None,
        btc_address: None,
        btc_price: None,
        transaction_hash: None,
    };

    services::payments::create(params.currencies, payload, state.postgres.clone()).and_then(
        move |payment| {
            payment
                .item(state.postgres.clone())
                .from_err()
                .and_then(move |item| {
                    JWTPayload::new(None, Some(auth_client))
                // TODO: Set expiration etc.
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
    let state = state.clone();
    let id = path.into_inner();

    services::payments::get(id, state.postgres.clone()).and_then(move |payment| {
        validate_client(&payment, &client)
            .into_future()
            .and_then(move |_| {
                services::vouchers::create(payment.clone(), state.postgres.clone()).then(
                    move |res| match res {
                        Ok(voucher) => {
                            Ok(Json(json!({
                                    "status": payment.status,
                                    "voucher": voucher,
                                })))
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            Ok(Json(json!({
                                    "status": payment.status,
                                })))
                        }
                    },
                )
            })
    })
}

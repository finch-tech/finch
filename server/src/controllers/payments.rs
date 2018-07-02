use std::collections::HashSet;

use actix_web::{Json, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;

use auth::{AuthClient, JWTPayload};
use core::client_token::ClientToken;
use core::payment::PaymentPayload;
use server::AppState;
use services::{self, Error};
use types::Currency;

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub currencies: HashSet<Currency>,
    pub amount: i32,
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
        amount: params.amount,
        store_id: auth_client.store_id,
        created_by: auth_client.id,
        created_at: None,
        paid_at: None,
        index: None,
        eth_address: None,
        btc_address: None,
    };

    services::payments::create(params.currencies, payload, state.postgres.clone()).and_then(
        move |payment| {
            JWTPayload::new(None, Some(auth_client))
                // TODO: Set expiration etc.
                .encode(&state.jwt_private)
                .map_err(|e| Error::from(e))
                .into_future()
                .then(move |res| {
                    res.and_then(|auth_token| {
                        Ok(Json(json!({
                            "payment": payment.export(),
                            "token": auth_token
                        })))
                    })
                })
        },
    )
}

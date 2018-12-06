use std::str::FromStr;

use actix_web::{Json, Path, State};
use bigdecimal::BigDecimal;
use futures::future::{self, err, ok, Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::app_status::AppStatus;
use core::client_token::ClientToken;
use core::payment::{Payment, PaymentPayload};
use server::AppState;
use services::{self, Error};
use types::{Currency, H160, U128, U256};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub currency: Currency,
    pub price: BigDecimal,
}

pub fn create(
    (state, client_token, params): (State<AppState>, ClientToken, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::stores::get(client_token.store_id, &state.postgres)
        .and_then(move |store| {
            if !store.can_accept(&params.currency) {
                return err(Error::CurrencyNotSupported);
            }

            ok((store, params))
        })
        .and_then(move |(store, params)| {
            let auth_client = AuthClient::new(client_token);

            let payload = PaymentPayload {
                status: None,
                store_id: auth_client.store_id,
                created_by: auth_client.id,
                created_at: None,
                expires_at: None,
                paid_at: None,
                index: None,
                base_price: Some(params.price),
                typ: Some(params.currency),
                address: None,
                price: None,
                confirmations_required: None,
                block_height_required: None,
                transaction_hash: None,
            };

            services::payments::create(
                payload,
                &state.postgres,
                state.config.currency_api_client.clone(),
            )
            .and_then(move |payment| {
                JWTPayload::new(None, Some(auth_client), payment.expires_at)
                    .encode(&state.config.jwt_private)
                    .map_err(|e| Error::from(e))
                    .into_future()
                    .then(move |res| {
                        res.and_then(|auth_token| {
                            Ok(Json(json!({
                                "payment": payment.export(),
                                "store": {
                                    "name": store.name,
                                    "description": store.description
                                },
                                "token": auth_token,
                            })))
                        })
                    })
            })
        })
}

fn validate_client(payment: &Payment, client: &AuthClient) -> Result<bool, Error> {
    if payment.created_by != client.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn get_eth_payment_status(
    state: State<AppState>,
    block_height: U128,
    payment: Payment,
) -> impl Future<Item = Json<Value>, Error = Error> {
    state
        .config
        .eth_rpc_client
        .get_balance(H160::from_str(&payment.address.clone().unwrap()[2..]).unwrap())
        .from_err()
        .map(move |balance| balance > U256::from(0))
        .and_then(move |payment_detected| {

            match payment.block_height_required {
                Some(block_height_required) => Ok(Json(json!({
                    "payment_detected": payment_detected,
                    "status": payment.status,
                    "confirmations_required": format!("{}", payment.confirmations_required.unwrap()),
                    "block_height": format!("{}", block_height),
                    "block_height_required": format!("{}", block_height_required)
                }))),
                None => Ok(Json(json!({
                    "payment_detected": payment_detected,
                    "status": payment.status,
                    "eth_confirmations_required": format!("{}", payment.confirmations_required.unwrap()),
                    "block_height": format!("{}", block_height),
                }))),
            }
        })
}

pub fn get_btc_payment_status(
    block_height: U128,
    payment: Payment,
) -> impl Future<Item = Json<Value>, Error = Error> {
    match payment.block_height_required {
        Some(block_height_required) => future::ok(Json(json!({
            "payment_detected": true,
            "status": payment.status,
            "confirmations_required": format!("{}", payment.confirmations_required.unwrap()),
            "block_height": format!("{}", block_height),
            "block_height_required": format!("{}", block_height_required)
        }))),
        None => future::ok(Json(json!({
            "payment_detected": false,
            "status": payment.status,
            "eth_confirmations_required": format!("{}", payment.confirmations_required.unwrap()),
            "block_height": format!("{}", block_height),
        }))),
    }
}

pub fn get_status(
    (state, client, path): (State<AppState>, AuthClient, Path<Uuid>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let id = path.into_inner();
    let app_status = AppStatus::find(&state.postgres).from_err();
    let payment = services::payments::get(id, &state.postgres).and_then(move |payment| {
        validate_client(&payment, &client)
            .into_future()
            .and_then(move |_| future::ok(payment))
    });

    app_status.join(payment).and_then(move |(status, payment)| {
        if let Some(block_height) = status.block_height(payment.typ) {
            if payment.address.is_none() {
                return future::Either::B(future::err(Error::CurrencyNotSupported));
            }

            let res = match payment.typ {
                Currency::Btc => future::Either::A(get_btc_payment_status(block_height, payment)),
                Currency::Eth => {
                    future::Either::B(get_eth_payment_status(state, block_height, payment))
                }
                _ => panic!("Invalid currency."),
            };
            future::Either::A(res)
        } else {
            return future::Either::B(future::err(Error::InternalServerError));
        }
    })
}

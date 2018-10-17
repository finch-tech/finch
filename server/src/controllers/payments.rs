use std::collections::HashSet;
use std::env;

use actix_web::{Json, Path, State};
use bigdecimal::BigDecimal;
use futures::future;
use futures::future::{err, ok, Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::app_status::AppStatus;
use core::client_token::ClientToken;
use core::payment::{Payment, PaymentPayload};
use ethereum_client::Client;
use server::AppState;
use services::{self, Error};
use types::{Currency, U256};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub currencies: HashSet<Currency>,
    pub price: BigDecimal,
}

pub fn create(
    (state, client_token, params): (State<AppState>, ClientToken, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();
    // TODO: Check params.currencies length.

    services::stores::get(client_token.store_id, &state.postgres)
        .and_then(move |store| {
            for (_, currency) in params.currencies.iter().enumerate() {
                if !store.can_accept(currency) {
                    return err(Error::CurrencyNotSupported);
                }
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
                price: Some(params.price),
                eth_address: None,
                eth_price: None,
                btc_address: None,
                btc_price: None,
                eth_confirmations_required: store.eth_confirmations_required.clone().unwrap(),
                eth_block_height_required: None,
                transaction_hash: None,
            };

            services::payments::create(params.currencies, payload, &state.postgres).and_then(
                move |payment| {
                    JWTPayload::new(None, Some(auth_client), payment.expires_at)
                        .encode(&state.jwt_private)
                        .map_err(|e| Error::from(e))
                        .into_future()
                        .then(move |res| {
                            res.and_then(|auth_token| {
                                Ok(Json(json!({
                                "payment": payment.export(),
                                "store": store.export(),
                                "token": auth_token,
                            })))
                            })
                        })
                },
            )
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
    let ethereum_rpc_url =
        env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL environment variable must be set.");

    let app_status = AppStatus::find(&state.postgres).from_err();
    let payment = services::payments::get(id, &state.postgres).and_then(move |payment| {
        validate_client(&payment, &client)
            .into_future()
            .and_then(move |_| future::ok(payment))
    });

    app_status.join(payment).and_then(move |(status, payment)| {
        if let Some(block_height) = status.eth_block_height {
            if let Some(eth_address) = payment.eth_address.clone() {
                return future::Either::B(
                    Client::new(ethereum_rpc_url)
                        .get_balance(eth_address)
                        .from_err()
                        .and_then(move |balance| {
                            let detected = balance > U256::from(0);

                            match payment.eth_block_height_required {
                                Some(eth_block_height_required) => Ok(Json(json!({
                                    "payment_detected": detected,
                                    "status": payment.status,
                                    "eth_confirmations_required": format!("{}", payment.eth_confirmations_required),
                                    "block_height": format!("{}", block_height),
                                    "eth_block_height_required": format!("{}", eth_block_height_required)
                                }))),
                                None => Ok(Json(json!({
                                    "payment_detected": detected,
                                    "status": payment.status,
                                    "eth_confirmations_required": format!("{}", payment.eth_confirmations_required),
                                    "block_height": format!("{}", block_height),
                                }))),
                            }
                        }),
                );
            } else {
                return future::Either::A(future::err(Error::CurrencyNotSupported));
            }
        } else {
            return future::Either::A(future::err(Error::InternalServerError));
        }
    })
}

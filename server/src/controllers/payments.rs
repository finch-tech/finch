use actix_web::{Json, Path, State};
use bigdecimal::BigDecimal;
use futures::future::{self, err, ok, Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::{
    app_status::AppStatus,
    client_token::ClientToken,
    payment::{Payment, PaymentPayload},
};
use server::AppState;
use services::{self, Error};
use types::{Currency, PaymentStatus, U128};

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
                store_id: Some(auth_client.store_id),
                created_by: Some(auth_client.id),
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

            let mut remaining_confirmations = U128::from(payment.confirmations_required.unwrap());

            if payment.status == PaymentStatus::Paid && payment.confirmations_required.unwrap() == 0
            {
                remaining_confirmations = U128::from(0);
            }

            match payment.block_height_required {
                Some(block_height_required) => {
                    if block_height_required < block_height {
                        remaining_confirmations = U128::from(0);
                    } else {
                        remaining_confirmations = block_height_required - block_height;
                    }
                }
                None => (),
            };

            let res = future::ok(Json(json!({
                "status": payment.status,
                "confirmations_required": format!("{}", payment.confirmations_required.unwrap()),
                "remaining_confirmations": remaining_confirmations,
            })));

            future::Either::A(res)
        } else {
            return future::Either::B(future::err(Error::InternalServerError));
        }
    })
}

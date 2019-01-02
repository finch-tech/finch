use actix_web::{Json, Path, State};
use bigdecimal::BigDecimal;
use futures::future::{self, err, ok, Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::{AuthClient, JWTPayload};
use core::{
    bitcoin::BlockchainStatus as BtcBlockchainStatus,
    client_token::ClientToken,
    ethereum::BlockchainStatus as EthBlockchainStatus,
    payment::{Payment, PaymentPayload},
};
use services::{self, Error};
use state::AppState;
use types::{
    currency::{Crypto, Fiat},
    PaymentStatus, U128,
};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub crypto: Crypto,
    pub fiat: Fiat,
    pub price: BigDecimal,
}

pub fn create(
    (state, client_token, params): (State<AppState>, ClientToken, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::stores::get(client_token.store_id, &state.postgres)
        .and_then(move |store| {
            if !store.can_accept(&params.crypto) {
                return err(Error::CurrencyNotSupported);
            }

            ok((store, params))
        })
        .and_then(move |(store, params)| {
            let auth_client = AuthClient::new(client_token);

            let mut payload = PaymentPayload::new();
            payload.store_id = Some(auth_client.store_id);
            payload.created_by = Some(auth_client.id);
            payload.fiat = Some(params.fiat);
            payload.price = Some(params.price);
            payload.crypto = Some(params.crypto);

            match params.crypto {
                Crypto::Btc => {
                    payload.confirmations_required = store.btc_confirmations_required;
                    payload.btc_network = state.btc_network;
                }
                Crypto::Eth => {
                    payload.confirmations_required = store.eth_confirmations_required;
                    payload.eth_network = state.eth_network;
                }
            }

            services::payments::create(
                payload,
                &store,
                &state.postgres,
                state.currency_api_client.clone(),
                state.btc_network,
            )
            .and_then(move |payment| {
                JWTPayload::new(None, Some(auth_client), payment.expires_at)
                    .encode(&state.jwt_private)
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

    services::payments::get(id, &state.postgres)
        .and_then(move |payment| {
            validate_client(&payment, &client)
                .into_future()
                .and_then(move |_| future::ok(payment))
        })
        .and_then(move |payment| {
            let block_height_future: Box<Future<Item = U128, Error = Error>> = match payment.crypto
            {
                Crypto::Btc => Box::new(
                    BtcBlockchainStatus::find(payment.btc_network.unwrap(), &state.postgres)
                        .from_err()
                        .map(move |status| status.block_height),
                ),
                Crypto::Eth => Box::new(
                    EthBlockchainStatus::find(payment.eth_network.unwrap(), &state.postgres)
                        .from_err()
                        .map(move |status| status.block_height),
                ),
            };

            block_height_future.and_then(move |block_height| {
                let mut remaining_confirmations = U128::from(payment.confirmations_required);

                if payment.status == PaymentStatus::Paid && payment.confirmations_required == 0 {
                    remaining_confirmations = U128::from(0);
                }

                if let Some(block_height_required) = payment.block_height_required {
                    if block_height_required < block_height {
                        remaining_confirmations = U128::from(0);
                    } else {
                        remaining_confirmations = block_height_required - block_height;
                    }
                }

                future::ok(Json(json!({
                    "status": payment.status,
                    "confirmations_required": payment.confirmations_required,
                    "remaining_confirmations": remaining_confirmations,
                })))
            })
        })
}

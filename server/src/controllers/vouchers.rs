use actix_web::{Json, State};
use futures::future::{Future, IntoFuture};
use serde_json::Value;
use uuid::Uuid;

use auth::AuthClient;
use core::payment::Payment;
use server::AppState;
use services::{self, Error};

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub payment_id: Uuid,
}

fn validate_client(payment: &Payment, client: &AuthClient) -> Result<bool, Error> {
    if payment.created_by != client.id {
        return Err(Error::InvalidRequestAccount);
    }

    Ok(true)
}

pub fn create(
    (state, client, params): (State<AppState>, AuthClient, Json<CreateParams>),
) -> impl Future<Item = Json<Value>, Error = Error> {
    let params = params.into_inner();

    services::payments::get(params.payment_id, &state.postgres).and_then(move |payment| {
        validate_client(&payment, &client)
            .into_future()
            .and_then(move |_| {
                services::vouchers::create(payment.clone(), &state.postgres).then(move |res| {
                    match res {
                        Ok(voucher) => Ok(Json(json!({
                                "status": payment.status,
                                "voucher": voucher,
                            }))),
                        Err(_) => Ok(Json(json!({
                            "status": payment.status,
                        }))),
                    }
                })
            })
    })
}

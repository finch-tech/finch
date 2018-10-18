use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse};
use futures::future::ok;

use server::AppState;

pub fn index(_: (HttpRequest<AppState>)) -> FutureResponse<HttpResponse> {
    ok(HttpResponse::Ok().json(json!({
        "name": "Payment Gateway Server",
        "version": "0.0.1"
    })))
    .responder()
}

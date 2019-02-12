use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse};
use futures::future::ok;

use state::AppState;

pub fn index(_: (HttpRequest<AppState>)) -> FutureResponse<HttpResponse> {
    ok(HttpResponse::Ok().json(json!({
        "name": "Finch Cryptocurrency Payment Processor",
        "version": "0.1.0"
    })))
    .responder()
}

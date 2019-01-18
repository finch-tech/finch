use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use serde_json::Error as SerdeError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "response error")]
    ResponseError,
    #[fail(display = "{}", _0)]
    SerdeError(#[cause] SerdeError),
    #[fail(display = "{}", _0)]
    SendRequestError(#[cause] SendRequestError),
    #[fail(display = "{}", _0)]
    PayloadError(#[cause] PayloadError),
}

impl From<SerdeError> for Error {
    fn from(e: SerdeError) -> Error {
        Error::SerdeError(e)
    }
}

impl From<SendRequestError> for Error {
    fn from(e: SendRequestError) -> Error {
        Error::SendRequestError(e)
    }
}

impl From<PayloadError> for Error {
    fn from(e: PayloadError) -> Error {
        Error::PayloadError(e)
    }
}

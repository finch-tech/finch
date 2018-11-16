use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use secp256k1::Error as Secp256k1Error;
use serde_json::Error as SerdeError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Response error")]
    ResponseError,
    #[fail(display = "Empty response error")]
    EmptyResponseError,
    #[fail(display = "{}", _0)]
    SerdeError(#[cause] SerdeError),
    #[fail(display = "{}", _0)]
    SendRequestError(#[cause] SendRequestError),
    #[fail(display = "{}", _0)]
    PayloadError(#[cause] PayloadError),
    #[fail(display = "{}", _0)]
    Secp256k1Error(#[cause] Secp256k1Error),
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

impl From<Secp256k1Error> for Error {
    fn from(e: Secp256k1Error) -> Error {
        Error::Secp256k1Error(e)
    }
}

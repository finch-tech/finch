use rpc_client::errors::Error as RpcClientError;

use actix::MailboxError;
use core::ModelError;
use hd_keyring::Error as KeyringError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "response error")]
    ResponseError,
    #[fail(display = "payment iteration error")]
    PaymentIterationError(Vec<Error>),
    #[fail(display = "{}", _0)]
    KeyringError(#[cause] KeyringError),
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    RpcClientError(#[cause] RpcClientError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "no payout address")]
    NoPayoutAddress,
    #[fail(display = "invalid gas price")]
    InvalidGasPrice,
}

impl From<KeyringError> for Error {
    fn from(e: KeyringError) -> Error {
        Error::KeyringError(e)
    }
}

impl From<ModelError> for Error {
    fn from(e: ModelError) -> Error {
        Error::ModelError(e)
    }
}

impl From<RpcClientError> for Error {
    fn from(e: RpcClientError) -> Error {
        Error::RpcClientError(e)
    }
}

impl From<MailboxError> for Error {
    fn from(e: MailboxError) -> Error {
        Error::MailboxError(e)
    }
}

use rpc_client::ethereum::Error as EthRpcClientError;

use actix::MailboxError;
use core::ModelError;
use hd_keyring::Error as KeyringError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Response error")]
    ResponseError,
    #[fail(display = "Payment iteration error")]
    PaymentIterationError(Vec<Error>),
    #[fail(display = "{}", _0)]
    KeyringError(#[cause] KeyringError),
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    EthRpcClientError(#[cause] EthRpcClientError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "No payout address")]
    NoPayoutAddress,
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

impl From<EthRpcClientError> for Error {
    fn from(e: EthRpcClientError) -> Error {
        Error::EthRpcClientError(e)
    }
}

impl From<MailboxError> for Error {
    fn from(e: MailboxError) -> Error {
        Error::MailboxError(e)
    }
}

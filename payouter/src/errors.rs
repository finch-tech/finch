use ethereum_client::Error as EthError;

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
    EthError(#[cause] EthError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
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

impl From<EthError> for Error {
    fn from(e: EthError) -> Error {
        Error::EthError(e)
    }
}

impl From<MailboxError> for Error {
    fn from(e: MailboxError) -> Error {
        Error::MailboxError(e)
    }
}

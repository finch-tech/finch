use std::io::Error as IoError;

use actix::MailboxError;
use core::ModelError;

use ethereum_client::Error as EthereumClientError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "{}", _0)]
    EthereumClientError(#[cause] EthereumClientError),
    #[fail(display = "{}", _0)]
    IoError(#[cause] IoError),
}

impl From<ModelError> for Error {
    fn from(e: ModelError) -> Error {
        Error::ModelError(e)
    }
}

impl From<MailboxError> for Error {
    fn from(e: MailboxError) -> Error {
        Error::MailboxError(e)
    }
}

impl From<EthereumClientError> for Error {
    fn from(e: EthereumClientError) -> Error {
        Error::EthereumClientError(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

use std::io::Error as IoError;

use actix::MailboxError;
use core::ModelError;

use eth_rpc_client::Error as EthRpcClientError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Exceeded retry limit: {}", _0)]
    RetryLimitError(i8),
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "{}", _0)]
    EthRpcClientError(#[cause] EthRpcClientError),
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

impl From<EthRpcClientError> for Error {
    fn from(e: EthRpcClientError) -> Error {
        Error::EthRpcClientError(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

use std::io::Error as IoError;

use actix::MailboxError;
use core::ModelError;

use rpc_client::errors::Error as RpcClientError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Exceeded retry limit: {}", _0)]
    RetryLimitError(usize),
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "{}", _0)]
    RpcClientError(#[cause] RpcClientError),
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

impl From<RpcClientError> for Error {
    fn from(e: RpcClientError) -> Error {
        Error::RpcClientError(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

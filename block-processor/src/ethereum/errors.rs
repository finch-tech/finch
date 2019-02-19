use std::io::Error as IoError;

use actix::MailboxError;
use core::ModelError;

use blockchain_api_client::errors::Error as BlockchainApiClientError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "exceeded retry limit: {}", _0)]
    RetryLimitError(usize),
    #[fail(display = "{}", _0)]
    ModelError(#[cause] ModelError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "{}", _0)]
    BlockchainApiClientError(#[cause] BlockchainApiClientError),
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

impl From<BlockchainApiClientError> for Error {
    fn from(e: BlockchainApiClientError) -> Error {
        Error::BlockchainApiClientError(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

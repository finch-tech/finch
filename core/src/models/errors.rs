use actix::MailboxError;
use db::Error as DbError;
use jwt::errors::Error as JwtError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    DbError(#[cause] DbError),
    #[fail(display = "{}", _0)]
    MailboxError(#[cause] MailboxError),
    #[fail(display = "{}", _0)]
    JwtError(#[cause] JwtError),
    #[fail(display = "property not found")]
    PropertyNotFound,
}

impl From<DbError> for Error {
    fn from(e: DbError) -> Error {
        Error::DbError(e)
    }
}

impl From<MailboxError> for Error {
    fn from(e: MailboxError) -> Error {
        Error::MailboxError(e)
    }
}

impl From<JwtError> for Error {
    fn from(e: JwtError) -> Error {
        Error::JwtError(e)
    }
}

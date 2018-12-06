use diesel::result::Error as DieselError;
use r2d2::Error as PoolError;
use serde_json::Error as SerdeJsonError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    DieselError(#[cause] DieselError),
    #[fail(display = "{}", _0)]
    PoolError(#[cause] PoolError),
    #[fail(display = "{}", _0)]
    SerdeJsonError(#[cause] SerdeJsonError),
}

impl From<DieselError> for Error {
    fn from(e: DieselError) -> Error {
        Error::DieselError(e)
    }
}

impl From<PoolError> for Error {
    fn from(e: PoolError) -> Error {
        Error::PoolError(e)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(e: SerdeJsonError) -> Error {
        Error::SerdeJsonError(e)
    }
}

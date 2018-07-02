use diesel::result::Error as DieselError;
use r2d2::Error as PoolError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    DieselError(#[cause] DieselError),
    #[fail(display = "{}", _0)]
    PoolError(#[cause] PoolError),
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

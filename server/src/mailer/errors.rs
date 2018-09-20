use lettre::smtp::error::Error as LettreError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    LettreError(#[cause] LettreError),
}

impl From<LettreError> for Error {
    fn from(e: LettreError) -> Error {
        Error::LettreError(e)
    }
}

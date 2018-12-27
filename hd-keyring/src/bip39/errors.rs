use std::num::ParseIntError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid word length {}", _0)]
    InvalidWordLength(usize),
    #[fail(display = "{}", _0)]
    ParseIntError(#[cause] ParseIntError),
    #[fail(display = "Invalid word {} provided", _0)]
    InvalidWord(String),
    #[fail(display = "{}", _0)]
    CustomError(String),
    #[fail(display = "Invalid checksum")]
    InvalidChecksum,
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

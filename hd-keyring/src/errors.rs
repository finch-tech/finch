use bip39::Error as Bip39Error;
use secp256k1::Error as Secp256k1Error;
use std::io::Error as IoError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Bip39Error(#[cause] Bip39Error),
    #[fail(display = "{}", _0)]
    Secp256k1Error(#[cause] Secp256k1Error),
    #[fail(display = "invalid derivation path")]
    InvalidDerivationPath,
    #[fail(display = "invalid derivation")]
    InvalidDerivation,
    #[fail(display = "invalid key length")]
    InvalidKeyLength,
    #[fail(display = "invalid base58 byte")]
    InvalidBase58Byte,
    #[fail(display = "bad checksum")]
    BadChecksum,
    #[fail(display = "IO error")]
    IoError(#[cause] IoError),
    #[fail(display = "invalid network")]
    InvalidNetwork,
}

impl From<Bip39Error> for Error {
    fn from(e: Bip39Error) -> Error {
        Error::Bip39Error(e)
    }
}

impl From<Secp256k1Error> for Error {
    fn from(e: Secp256k1Error) -> Error {
        Error::Secp256k1Error(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::IoError(e)
    }
}

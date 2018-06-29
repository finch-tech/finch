extern crate bigdecimal;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
extern crate digest;
extern crate num_traits;
extern crate ripemd160;
extern crate rustc_hex;
extern crate serde;
extern crate sha2;
#[macro_use]
extern crate serde_derive;
extern crate web3;

mod clients;
mod currencies;
mod h160;
mod h256;
mod payment_status;
mod u256;

pub type PrivateKey = Vec<u8>;
pub type PublicKey = Vec<u8>;

pub use self::clients::Client;
pub use self::currencies::Currency;
pub use self::h160::H160;
pub use self::h256::H256;
pub use self::payment_status::Status;
pub use self::u256::U256;

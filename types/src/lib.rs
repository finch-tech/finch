#![allow(proc_macro_derive_resolution_fallback)]

extern crate bigdecimal;
extern crate byteorder;
extern crate core;
#[macro_use]
extern crate crunchy;
#[macro_use]
extern crate diesel;
extern crate digest;
extern crate ethereum_types;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate ripemd160;
extern crate rlp;
extern crate rust_base58;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
#[macro_use]
extern crate uint;

pub mod bitcoin;
mod clients;
pub mod currency;
pub mod ethereum;
mod h160;
mod h256;
mod payment_status;
mod payout_actions;
mod payout_status;
mod u128;
mod u256;

pub type PrivateKey = Vec<u8>;
pub type PublicKey = Vec<u8>;

pub use self::clients::Client;
pub use self::h160::H160;
pub use self::h256::H256;
pub use self::payment_status::PaymentStatus;
pub use self::payout_actions::PayoutAction;
pub use self::payout_status::PayoutStatus;
pub use self::u128::U128;
pub use self::u256::U256;

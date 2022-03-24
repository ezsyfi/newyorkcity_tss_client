#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

pub mod btc;
pub mod ecdsa;
pub mod escrow;
pub mod eth;
pub mod wallet;
pub mod utilities;

// pub mod eddsa;
// pub mod schnorr;

mod tests;

type Result<T> = std::result::Result<T, failure::Error>;

pub use curv::{arithmetic::traits::Converter, BigInt};
// pub use multi_party_eddsa::protocols::aggsig::*;

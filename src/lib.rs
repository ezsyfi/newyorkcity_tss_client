#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;

pub mod btc;
pub mod ecdsa;
pub mod escrow;
pub mod eth;
pub mod utilities;
pub mod wallet;

// pub mod eddsa;
// pub mod schnorr;

mod tests;



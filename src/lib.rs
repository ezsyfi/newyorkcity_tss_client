#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;


pub mod ecdsa;
pub mod btc;
pub mod escrow;
pub mod wallet;

// pub mod eddsa;
pub mod schnorr;

mod utilities;
mod tests;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub struct ClientShim {
    pub client: reqwest::blocking::Client,
    pub auth_token: Option<String>,
    pub endpoint: String,
}

impl ClientShim {
    pub fn new(endpoint: String, auth_token: Option<String>) -> ClientShim {
        let client = reqwest::blocking::Client::new();
        ClientShim {
            client,
            auth_token,
            endpoint,
        }
    }
}

pub use curv::{BigInt, arithmetic::traits::Converter};
// pub use multi_party_eddsa::protocols::aggsig::*;

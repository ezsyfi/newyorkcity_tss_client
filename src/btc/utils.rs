use bitcoin::{self, Network};
use curv::elliptic::curves::secp256_k1::PK;
use curv::elliptic::curves::traits::ECPoint;
use kms::ecdsa::two_party::MasterKey2;
use curv::BigInt;

use crate::ecdsa::PrivateShare;

pub const BTC_TESTNET: &str = "testnet";

pub fn to_bitcoin_address(network: String, mk: &MasterKey2) -> bitcoin::Address {
    let pk = mk.public.q.get_element();
    let address = bitcoin::Address::p2wpkh(
        &to_bitcoin_public_key(pk),
        network.parse::<bitcoin::Network>().unwrap(),
    )
    .expect("Cannot panic because `to_bitcoin_public_key` creates a compressed address");
    address
}

pub fn get_new_bitcoin_address(private_share: &PrivateShare, last_derived_pos: u32) -> bitcoin::Address {
    let (_pos, mk) = derive_new_key(&private_share, last_derived_pos);
    to_bitcoin_address(BTC_TESTNET.to_owned(), &mk)
}

pub fn to_bitcoin_public_key(pk: PK) -> bitcoin::util::key::PublicKey {
    bitcoin::util::key::PublicKey {
        compressed: true,
        key: pk,
    }
}


pub fn derive_new_key(private_share: &PrivateShare, pos: u32) -> (u32, MasterKey2) {
    let last_pos: u32 = pos + 1;

    let last_child_master_key = private_share
        .master_key
        .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

    (last_pos, last_child_master_key)
}

pub fn get_bitcoin_network() -> Network {
    BTC_TESTNET.to_owned().parse::<Network>().unwrap()
}
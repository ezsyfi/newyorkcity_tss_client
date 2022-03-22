use curv::elliptic::curves::traits::ECPoint;
use kms::ecdsa::two_party::MasterKey2;
use web3::{self, signing::keccak256, types::Address};

pub fn to_eth_address(mk: &MasterKey2) -> Address {
    let pub_k = mk.public.q.get_element().serialize_uncompressed();
    let hash = keccak256(&pub_k[1..]);
    Address::from_slice(&hash[12..])
}

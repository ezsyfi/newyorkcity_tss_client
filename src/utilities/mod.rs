pub mod a_requests;
pub mod dto;
pub mod err_handling;
pub mod ffi;
pub mod requests;
pub mod tests;
pub mod key_recover;

use crate::ecdsa::PrivateShare;
use kms::ecdsa::two_party::MasterKey2;
use two_party_ecdsa::BigInt;

pub fn derive_new_key(private_share: &PrivateShare, pos: u32) -> (u32, MasterKey2) {
    let last_pos: u32 = pos + 1;

    let last_child_master_key = private_share
        .master_key
        .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

    (last_pos, last_child_master_key)
}

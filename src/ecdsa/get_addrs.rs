// iOS bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use bitcoin::Network;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use curv::elliptic::curves::traits::ECPoint;
use curv::elliptic::curves::secp256_k1::PK;
use super::PrivateShare;

#[derive(Serialize, Deserialize)]
struct GetBtcAddressFFIResp {
    address: String,
    pos: i32,
    master_key: MasterKey2
}

fn derive_new_key(private_share: &PrivateShare, pos: i32) -> (i32, MasterKey2) {
    let last_pos: i32 = pos + 1;

    let last_child_master_key = private_share
        .master_key
        .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

    (last_pos, last_child_master_key)
}

fn to_bitcoin_public_key(pk: PK) -> bitcoin::util::key::PublicKey {
    bitcoin::util::key::PublicKey {
        compressed: true,
        key: pk,
    }
}

#[no_mangle]
pub extern "C" fn get_btc_addrs(
    c_private_share_json: *const c_char,
    c_last_derived_pos: i32,
) -> *mut c_char {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw private share"),
    };
    let private_share: PrivateShare = serde_json::from_str(&private_share_json).unwrap();

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);
    let pk = mk.public.q.get_element();
    let network = "testnet".to_owned();
    let address = bitcoin::Address::p2wpkh(
        &to_bitcoin_public_key(pk),
        network.parse::<Network>().unwrap(),
    )
    .expect("Cannot panic because `to_bitcoin_public_key` creates a compressed address");

    let get_addr_resp = GetBtcAddressFFIResp {
        address: address.to_string(),
        pos: pos,
        master_key: mk
    };

    let get_addr_resp_json = match serde_json::to_string(&get_addr_resp) {
        Ok(share) => share,
        Err(_) => panic!("Error while performing get btc addrs"),
    };

    CString::new(get_addr_resp_json.to_owned())
        .unwrap()
        .into_raw()
}

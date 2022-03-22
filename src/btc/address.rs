// iOS bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ecdsa::PrivateShare;
use crate::utilities::dto::MKPosAddressFFI;
use crate::utilities::hd_wallet::derive_new_key;

use super::utils::{ BTC_TESTNET, to_bitcoin_address};

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_btc_addrs(
    c_private_share_json: *const c_char,
    c_last_derived_pos: u32,
) -> *mut c_char {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw private share"),
    };
    let private_share: PrivateShare = serde_json::from_str(private_share_json).unwrap();

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);
    let address = to_bitcoin_address(BTC_TESTNET, &mk);

    let get_addr_resp = MKPosAddressFFI {
        address: address.to_string(),
        pos,
        mk,
    };

    let get_addr_resp_json = match serde_json::to_string(&get_addr_resp) {
        Ok(addrs_resp) => addrs_resp,
        Err(_) => panic!("Error while performing get btc addrs"),
    };

    CString::new(get_addr_resp_json).unwrap().into_raw()
}

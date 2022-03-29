// iOS bindings
use crate::ecdsa::PrivateShare;
use crate::utilities::dto::MKPosAddressDto;
use crate::utilities::err_handling::error_to_c_string;
use crate::utilities::hd_wallet::derive_new_key;
use anyhow::anyhow;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use super::utils::{to_bitcoin_address, BTC_TESTNET};

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_btc_addrs(
    c_private_share_json: *const c_char,
    c_last_derived_pos: u32,
) -> *mut c_char {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(_) => return error_to_c_string(anyhow!("E102: parse private share to JSON failed")),
    };
    let private_share: PrivateShare = match serde_json::from_str(private_share_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(anyhow!(
                "E100: Error while deserializing private share: {}",
                e
            ))
        }
    };

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);

    let address = match to_bitcoin_address(BTC_TESTNET, &mk) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(anyhow!("E103: Error while creating btc address: {}", e))
        }
    };

    let mk_pos_address = MKPosAddressDto {
        address: address.to_string(),
        pos,
        mk,
    };

    let mk_pos_address_json = match serde_json::to_string(&mk_pos_address) {
        Ok(addrs_resp) => addrs_resp,
        Err(_) => return error_to_c_string(anyhow!("E102: parse MKPosAddressDTO to JSON failed")),
    };

    match CString::new(mk_pos_address_json) {
        Ok(s) => s.into_raw(),
        Err(_) => error_to_c_string(anyhow!("E101: Error while encoding mk,pos,address dto")),
    }
}

use crate::dto::ecdsa::MKPosAddressDto;
use crate::ecdsa::PrivateShare;
use crate::utilities::derive_new_key;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
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
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };
    let private_share: PrivateShare = match serde_json::from_str(private_share_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);

    let address = match to_bitcoin_address(BTC_TESTNET, &mk) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "bitcoin_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let mk_pos_address = MKPosAddressDto {
        address: address.to_string(),
        pos,
        mk,
    };

    let mk_pos_address_json = match serde_json::to_string(&mk_pos_address) {
        Ok(addrs_resp) => addrs_resp,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "mk_pos_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(mk_pos_address_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "mk_pos_address".to_owned(),
            e: e.to_string(),
        }),
    }
}

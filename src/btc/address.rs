// iOS bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ecdsa::PrivateShare;

use super::utils::{derive_new_key, get_new_bitcoin_address};
use kms::ecdsa::two_party::MasterKey2;


#[derive(Serialize, Deserialize)]
struct GetBtcAddressFFIResp {
    address: String,
    pos: u32,
    mk: MasterKey2,
}

#[no_mangle]
pub extern "C" fn get_btc_addrs(
    c_private_share_json: *const c_char,
    c_last_derived_pos: u32,
) -> *mut c_char {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw private share"),
    };
    let private_share: PrivateShare = serde_json::from_str(&private_share_json).unwrap();

    let (pos, mk) = derive_new_key(&private_share, c_last_derived_pos);
    let address = get_new_bitcoin_address(&private_share, c_last_derived_pos);

    let get_addr_resp = GetBtcAddressFFIResp {
        address: address.to_string(),
        pos: pos,
        mk: mk,
    };

    let get_addr_resp_json = match serde_json::to_string(&get_addr_resp) {
        Ok(share) => share,
        Err(_) => panic!("Error while performing get btc addrs"),
    };

    CString::new(get_addr_resp_json.to_owned())
        .unwrap()
        .into_raw()
}


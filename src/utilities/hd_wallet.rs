use crate::ecdsa::PrivateShare;
use anyhow::Result;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use std::ffi::CStr;
use std::os::raw::c_char;

use super::err_handling::ErrorFFIKind;

pub fn derive_new_key(private_share: &PrivateShare, pos: u32) -> (u32, MasterKey2) {
    let last_pos: u32 = pos + 1;

    let last_child_master_key = private_share
        .master_key
        .get_child(vec![BigInt::from(0), BigInt::from(last_pos)]);

    (last_pos, last_child_master_key)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::unnecessary_unwrap)]
pub fn get_private_share(c_private_share_json: *const c_char) -> Result<PrivateShare> {
    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return Err(ErrorFFIKind::E102 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            }
            .into());
        }
    };

    let private_share = serde_json::from_str(private_share_json);

    if private_share.is_ok() {
        Ok(private_share.unwrap())
    } else {
        Err(ErrorFFIKind::E104 {
            msg: "private_share".to_owned(),
            e: private_share.err().unwrap().to_string(),
        }
        .into())
    }
}

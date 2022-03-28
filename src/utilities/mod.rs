use std::ffi::CString;
use std::os::raw::c_char;

pub mod dto;
pub mod err_handling;
pub mod hd_wallet;
pub mod requests;

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cstring_free(cstring: *mut c_char) {
    if cstring.is_null() {
        return;
    }
    unsafe { CString::from_raw(cstring) };
}

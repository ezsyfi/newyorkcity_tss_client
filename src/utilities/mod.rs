use std::ffi::CString;
use std::os::raw::c_char;

pub mod dto;
pub mod hd_wallet;
pub mod requests;

pub fn error_to_c_string(e: failure::Error) -> *mut c_char {
    CString::new(format!("Error: {}", e)).unwrap().into_raw()
}

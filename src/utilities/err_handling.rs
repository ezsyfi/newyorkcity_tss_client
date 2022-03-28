use std::ffi::CString;
use std::os::raw::c_char;

use anyhow::Error;

pub fn error_to_c_string(e: Error) -> *mut c_char {
    CString::new(format!("{}", e)).unwrap().into_raw()
}

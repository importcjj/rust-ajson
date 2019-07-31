
extern crate gjson;
extern crate libc;
use libc::c_char;
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern "C" fn gjson_get(json: *const c_char, path: *const c_char) -> *mut c_char {
    let json = unsafe { CStr::from_ptr(json).to_str().unwrap() };
    let path = unsafe { CStr::from_ptr(path).to_str().unwrap() };

    let result = gjson::get(json, path);
    let s = CString::new(result.as_str()).unwrap();
    s.into_raw()
}

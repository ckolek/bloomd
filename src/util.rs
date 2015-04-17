extern crate libc;

use std::{ffi, mem};

// makes a call to strerror and returns an Err with the returned String value
pub fn strerror<T>(errnum : i32) -> Result<T, String> {
    let error : *const libc::c_char = unsafe { libc::funcs::c95::string::strerror(errnum) };

    if !error.is_null() {
        let buf : &[u8] = unsafe { ffi::c_str_to_bytes(&error) };
        let error_string : String = String::from_utf8(buf.to_vec()).unwrap();

        unsafe {
            libc::free(mem::transmute(error));
        }

        return Err(error_string);
    } else {
        return Err(String::new());
    }
}

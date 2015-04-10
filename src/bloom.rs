#![allow(unstable)]

extern crate libc;

use std::ffi;
use bitmap::bloom_bitmap;

#[repr(C, packed)]
pub struct bloom_filter_header {
    magic : u32,
    k_num : u32,
    count : u64,
    __buf : [u8; 496]
}

#[repr(C)]
pub struct bloom_bloomfilter<'a> {
    header      : bloom_filter_header,
    map         : bloom_bitmap<'a>,
    offset      : u64,
    bitmap_size : u64
}

impl<'a> bloom_bloomfilter<'a> {
    pub fn add(&mut self, key : String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::bf_add(self as *mut bloom_bloomfilter, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    pub fn contains(&self, key : &String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::bf_contains(self as *const bloom_bloomfilter, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    pub fn size(&self) -> u64 {
        return unsafe { externals::bf_size(self as *const bloom_bloomfilter) };
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::bf_flush(self as *mut bloom_bloomfilter) } < 0 {
            return Err(());
        } else {
            return Ok(());
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for bloom_bloomfilter<'a> {
    fn drop(&mut self) {
        drop(&mut self.map);

        unsafe { externals::bf_close(self as *mut bloom_bloomfilter) };
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong};
    use super::bloom_bloomfilter;
    use bitmap::bloom_bitmap;

    #[link(name = "bloom")]
    extern {
        pub fn bf_from_bitmap(map : *mut bloom_bitmap, k_num : c_uint, new_filter : c_int, filter : *mut bloom_bloomfilter) -> c_int;

        pub fn bf_add(filter : *mut bloom_bloomfilter, key : *const c_char) -> c_int;

        pub fn bf_contains(filter : *const bloom_bloomfilter, key : *const c_char) -> c_int;

        pub fn bf_size(filter : *const bloom_bloomfilter) -> c_ulong;

        pub fn bf_flush(filter : *mut bloom_bloomfilter) -> c_int;

        pub fn bf_close(filter : *mut bloom_bloomfilter) -> c_int;
    }
}

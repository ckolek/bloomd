#![allow(unstable)]

extern crate libc;

use bitmap::bloom_bitmap;

#[repr(C, packed)]
pub struct bloom_filter_header {
    magic : u32,
    k_num : u32,
    count : u64,
    __buf : [u8; 496]
}

#[repr(C)]
pub struct bloom_bloomfilter {
    header      : bloom_filter_header,
    map         : bloom_bitmap,
    offset      : u64,
    bitmap_size : u64
}

impl bloom_bloomfilter {
    pub fn add(&mut self, key : String) -> Result<bool, ()> {
        return Ok(false);
    }

    pub fn contains(&self, key : &String) -> Result<bool, ()> {
        return Ok(false);
    }

    pub fn size(&self) -> u64 {
        return 0;
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        return Ok(());
    }
}

impl Drop for bloom_bloomfilter {
    fn drop(&mut self) {
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

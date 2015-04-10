#![allow(unstable)]

extern crate libc;

use self::libc::{c_char, malloc, size_t};
use std::{mem, ffi, ptr};
use bitmap::bloom_bitmap;

#[repr(C, packed)]
pub struct bloom_filter_header {
    magic : u32,
    k_num : u32,
    count : u64,
    __buf : [u8; 496]
}

impl bloom_filter_header {
    pub fn new(magic : u32, k_num : u32, count : u64) -> Self {
        return bloom_filter_header { magic: magic, k_num: k_num, count: count, __buf: [0; 496] };
    }
}

#[repr(C)]
pub struct bloom_bloomfilter<'a> {
    header      : bloom_filter_header,
    map         : bloom_bitmap<'a>,
    offset      : u64,
    bitmap_size : u64
}

impl<'a> bloom_bloomfilter<'a> {
    pub fn new(mut map : bloom_bitmap<'a>, k_num : u32, new_filter : bool) -> Self {
        unsafe {
            let filter_ptr : *mut bloom_bloomfilter = malloc(mem::size_of::<bloom_bloomfilter>() as size_t) as *mut bloom_bloomfilter;

            externals::bf_from_bitmap(&mut map as *mut bloom_bitmap, k_num, new_filter as i32, filter_ptr);

            return ptr::read(filter_ptr);
        };
    }

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

#[repr(C)]
pub struct bloom_filter_params {
    bytes          : u64,
    k_num          : u32,
    capacity       : u64,
    fp_probability : f64
}

impl bloom_filter_params {
    fn empty() -> Self {
        return bloom_filter_params::new(0, 0, 0, 0.0);
    }

    fn new(bytes : u64, k_num : u32, capacity : u64, fp_probability : f64) -> Self {
        return bloom_filter_params { bytes: bytes, k_num: k_num, capacity: capacity, fp_probability: fp_probability };
    }
}

#[unsafe_destructor]
impl<'a> Drop for bloom_bloomfilter<'a> {
    fn drop(&mut self) {
        drop(&mut self.map);

        unsafe { externals::bf_close(self as *mut bloom_bloomfilter) };
    }
}

pub fn compute_hashes(k_num : u32, key : &str) -> u64 {
    let key : ffi::CString = ffi::CString::from_slice(key.as_bytes());

    let mut hashes : u64 = 0;

    unsafe { externals:: bf_compute_hashes(k_num, key.as_ptr() as *mut c_char, &mut hashes as *mut u64) };

    return hashes;
}

pub fn params_for_capacity(params : &mut bloom_filter_params) -> Result<(), ()> {
    if unsafe { externals::bf_params_for_capacity(params as *mut bloom_filter_params) } < 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

pub fn size_for_capacity_prob(params : &mut bloom_filter_params) -> Result<(), ()> {
    if unsafe { externals::bf_size_for_capacity_prob(params as *mut bloom_filter_params) } < 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

pub fn fp_probability_for_capacity_size(params : &mut bloom_filter_params) -> Result<(), ()> {
    if unsafe { externals::bf_fp_probability_for_capacity_size(params as *mut bloom_filter_params) } < 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

pub fn capacity_for_size_prob(params : &mut bloom_filter_params) -> Result<(), ()> {
    if unsafe { externals::bf_capacity_for_size_prob(params as *mut bloom_filter_params) } < 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

pub fn ideal_k_num(params : &mut bloom_filter_params) -> Result<(), ()> {
    if unsafe { externals::bf_ideal_k_num(params as *mut bloom_filter_params) } < 0 {
        return Err(());
    } else {
        return Ok(());
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong};
    use super::{bloom_bloomfilter, bloom_filter_params};
    use bitmap::bloom_bitmap;

    #[link(name = "bloom")]
    extern {
        pub fn bf_from_bitmap(map : *mut bloom_bitmap, k_num : c_uint, new_filter : c_int, filter : *mut bloom_bloomfilter) -> c_int;

        pub fn bf_add(filter : *mut bloom_bloomfilter, key : *const c_char) -> c_int;

        pub fn bf_contains(filter : *const bloom_bloomfilter, key : *const c_char) -> c_int;

        pub fn bf_size(filter : *const bloom_bloomfilter) -> c_ulong;

        pub fn bf_flush(filter : *mut bloom_bloomfilter) -> c_int;

        pub fn bf_close(filter : *mut bloom_bloomfilter) -> c_int;

        pub fn bf_compute_hashes(k_num : c_uint, key : *mut c_char, hashes : *mut c_ulong);

        pub fn bf_params_for_capacity(params : *mut bloom_filter_params) -> c_int;

        pub fn bf_size_for_capacity_prob(params : *mut bloom_filter_params) -> c_int;

        pub fn bf_fp_probability_for_capacity_size(params : *mut bloom_filter_params) -> c_int;

        pub fn bf_capacity_for_size_prob(params : *mut bloom_filter_params) -> c_int;

        pub fn bf_ideal_k_num(params : *mut bloom_filter_params) -> c_int;
    }
}

#[cfg(test)]
mod tests {
    use super::bloom_filter_params;

    #[test]
    fn test() {
        let mut params : bloom_filter_params = bloom_filter_params::empty();
        params.k_num = 1;
    }
}

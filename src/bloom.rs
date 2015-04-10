#![allow(unstable)]

extern crate libc;

use self::libc::c_char;
use std::ffi;
use bitmap::bloom_bitmap;
use filter::BloomFilter;

#[repr(C, packed)]
pub struct bloom_filter_header {
    magic : u32,
    k_num : u32,
    count : u64,
    __buf : [i8; 496]
}

impl bloom_filter_header {
    pub fn new(magic : u32, k_num : u32, count : u64) -> Self {
        return bloom_filter_header { magic: magic, k_num: k_num, count: count, __buf: [0; 496] };
    }
}

#[repr(C)]
pub struct bloom_bloomfilter<'a> {
    header      : Box<bloom_filter_header>,
    map         : Box<bloom_bitmap>,
    offset      : u64,
    bitmap_size : u64
}

impl<'a> bloom_bloomfilter<'a> {
    pub fn new(map : bloom_bitmap, k_num : u32, new_filter : bool) -> Self {
        let mut filter : bloom_bloomfilter = bloom_bloomfilter {
            header: Box::new(bloom_filter_header::new(0, k_num, 0)),
            map: Box::new(map),
            offset: 0,
            bitmap_size: 0
        };

        unsafe {
            externals::bf_from_bitmap(&mut *filter.map, k_num, new_filter as i32, &mut filter as *mut bloom_bloomfilter);
        };

        return filter;
    }
}

impl<'a> BloomFilter<bool> for bloom_bloomfilter<'a> {
    fn add(&mut self, key : String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());
        let result : i32 = unsafe { externals::bf_add(self as *mut bloom_bloomfilter, key.as_ptr()) };

        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    fn contains(&self, key : &String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::bf_contains(self as *const bloom_bloomfilter, key.as_ptr()) };

        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    fn size(&self) -> u64 {
        return unsafe { externals::bf_size(self as *const bloom_bloomfilter) };
    }

    fn flush(&mut self) -> Result<(), ()> {
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
        unsafe { externals::bf_close(self as *mut bloom_bloomfilter) };
    }
}

#[repr(C)]
pub struct bloom_filter_params {
    pub bytes          : u64,
    pub k_num          : u32,
    pub capacity       : u64,
    pub fp_probability : f64
}

impl bloom_filter_params {
    pub fn empty() -> Self {
        return bloom_filter_params::new(0, 0, 0, 0.0);
    }

    pub fn new(bytes : u64, k_num : u32, capacity : u64, fp_probability : f64) -> Self {
        return bloom_filter_params { bytes: bytes, k_num: k_num, capacity: capacity, fp_probability: fp_probability };
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
    #[link(name = "spooky")]
    #[link(name = "murmur")]
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
    use super::{bloom_bloomfilter, bloom_filter_params};
    use filter;

    static BITMAP_FILE : &'static str = "map.bmp";

    #[test]
    fn test() {
        let params : bloom_filter_params = filter::test::create_bloom_filter_params();
        let filter : bloom_bloomfilter = filter::test::create_bloom_filter(&params, BITMAP_FILE);

        filter::test::test_filter(Box::new(filter),
            &[[true, false, false], [false, true, false], [false, false, true]],
            &[[true, false, false], [true, true, false], [true, true, true]]);
    }
}

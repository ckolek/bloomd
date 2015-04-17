#![allow(unstable)]

extern crate libc;

use self::libc::c_char;
use std::ffi;
use bitmap::{bitmap_mode, bloom_bitmap};
use filter::IBloomFilter;
use util;

// Struct representing the header of a bloom filter
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

// Struct representing a bloom filter
#[repr(C)]
pub struct bloom_bloomfilter {
    header      : Box<bloom_filter_header>,
    map         : Box<bloom_bitmap>,
    offset      : u64,
    bitmap_size : u64
}

impl bloom_bloomfilter {
    // Creates a new bloom filter from the given bitmap
    pub fn new(k_num : u32, count : u64, map : bloom_bitmap, new_filter : bool) -> Result<Self, String> {
        let mut filter : bloom_bloomfilter = bloom_bloomfilter {
            header: Box::new(bloom_filter_header::new(if new_filter { 0 } else { externals::MAGIC_HEADER }, k_num, count)),
            map: Box::new(map),
            offset: 0,
            bitmap_size: 0
        };

        let value : i32 = unsafe { externals::bf_from_bitmap(&mut *filter.map, k_num, new_filter as i32, &mut filter as *mut bloom_bloomfilter) };

        if value < 0 {
            return util::strerror(value);
        }

        return Ok(filter);
    }
}

impl IBloomFilter<bool> for bloom_bloomfilter {
    // Adds a key to the filter
    fn add(&mut self, key : String) -> Result<bool, String> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());
        let result : i32 = unsafe { externals::bf_add(self as *mut bloom_bloomfilter, key.as_ptr()) };

        if result < 0 {
            return util::strerror(result);
        }

        return Ok(result > 0);
    }

    // Returns true if the key is probably in the filter, and false if it definitely isn't
    fn contains(&self, key : &String) -> Result<bool, String> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::bf_contains(self as *const bloom_bloomfilter, key.as_ptr()) };

        if result < 0 {
            return util::strerror(result);
        }

        return Ok(result > 0);
    }

    // Returns the number of keys in the filter
    fn size(&self) -> u64 {
        return unsafe { externals::bf_size(self as *const bloom_bloomfilter) };
    }

    // Flushes the filter to disk
    fn flush(&mut self) -> Result<(), String> {
        let value : i32 = unsafe { externals::bf_flush(self as *mut bloom_bloomfilter) };

        if value < 0 {
            return util::strerror(value);
        }

        return Ok(());
    }
}

// Closes the filter in C memory when the filter is dropped
impl Drop for bloom_bloomfilter {
    fn drop(&mut self) {
        unsafe { externals::bf_close(self as *mut bloom_bloomfilter) };
    }
}

// The parameters for a given bloom filter
#[repr(C)]
pub struct bloom_filter_params {
    pub bytes          : u64,
    pub k_num          : u32,
    pub capacity       : u64,
    pub fp_probability : f64
}

impl bloom_filter_params {
    // Returns an empty set of parameters
    pub fn empty() -> Self {
        return bloom_filter_params::new(0, 0, 0, 0.0);
    }

    // Returns a set of parameters with the values given
    pub fn new(bytes : u64, k_num : u32, capacity : u64, fp_probability : f64) -> Self {
        return bloom_filter_params { bytes: bytes, k_num: k_num, capacity: capacity, fp_probability: fp_probability };
    }
}

// Computes the hashes for a bloom filter
pub fn compute_hashes(k_num : u32, key : &str) -> u64 {
    let key : ffi::CString = ffi::CString::from_slice(key.as_bytes());

    let mut hashes : u64 = 0;

    unsafe { externals:: bf_compute_hashes(k_num, key.as_ptr() as *mut c_char, &mut hashes as *mut u64) };

    return hashes;
}

// Creates the bloom filter parameters, including the given capacity and probability
pub fn create_bloom_filter_params(capacity : u64, probability : f64) -> Result<bloom_filter_params, String> {
    let mut params : bloom_filter_params = bloom_filter_params::empty();
    params.capacity = capacity;
    params.fp_probability = probability;

    match size_for_capacity_prob(&mut params) {
        Err(e) => { return Err(e) },
        Ok(_) => { }
    }

    match ideal_k_num(&mut params) {
        Err(e) => { return Err(e) },
        Ok(_) => { }
    }

    return Ok(params);
}

// Creates a fresh bloom filter
pub fn create_bloom_filter(params : &bloom_filter_params, bitmap_filename : &str, in_memory : bool) -> Result<bloom_bloomfilter, String> {
    let mode : u32 = bitmap_mode::NEW_BITMAP | if in_memory { bitmap_mode::ANONYMOUS } else { bitmap_mode::PERSISTENT };

    let map : bloom_bitmap;
    match bloom_bitmap::from_filename(bitmap_filename, params.bytes, true, mode) {
        Ok(_map) => { map = _map },
        Err(e) => { return Err(e) }
    }

    return bloom_bloomfilter::new(params.k_num, 0, map, true);
}

// Loads the bloom filter with the bitmap from the given filename
pub fn load_bloom_filter(params : &bloom_filter_params, count : u64, bitmap_filename : &str, in_memory : bool) -> Result<bloom_bloomfilter, String> {
    let mode : u32 = if in_memory { bitmap_mode::ANONYMOUS } else { bitmap_mode::PERSISTENT } as u32;

    let map : bloom_bitmap;
    match bloom_bitmap::from_filename(bitmap_filename, params.bytes, false, mode) {
        Ok(_map) => { map = _map },
        Err(e) => { return Err(e) }
    }

    return bloom_bloomfilter::new(params.k_num, count, map, false);
}

// Expects capacity and probability to be set, and sets the bytes and k_num that should be used.
pub fn params_for_capacity(params : &mut bloom_filter_params) -> Result<(), String> {
    let value : i32 = unsafe { externals::bf_params_for_capacity(params as *mut bloom_filter_params) };

    if value < 0 {
        return util::strerror(value);
    }

    return Ok(());
}

// Expects capacity and probability to be set, computes the minimum byte size required.
pub fn size_for_capacity_prob(params : &mut bloom_filter_params) -> Result<(), String> {
    let value : i32 = unsafe { externals::bf_size_for_capacity_prob(params as *mut bloom_filter_params) };

    if value < 0 {
        return util::strerror(value);
    }

    return Ok(());
}

//  Expects capacity and size to be set, computes the best false positive probability given an ideal k.
pub fn fp_probability_for_capacity_size(params : &mut bloom_filter_params) -> Result<(), String> {
    let value : i32 = unsafe { externals::bf_fp_probability_for_capacity_size(params as *mut bloom_filter_params) };

    if value < 0 {
        return util::strerror(value);
    }

    return Ok(());
}

//Expects bytes and probability to be set, computes the expected capacity.
pub fn capacity_for_size_prob(params : &mut bloom_filter_params) -> Result<(), String> {
    let value : i32 = unsafe { externals::bf_capacity_for_size_prob(params as *mut bloom_filter_params) };

    if value < 0 {
        return util::strerror(value);
    }

    return Ok(());
}

// Expects bytes and capacity to be set, computes the ideal k num.
pub fn ideal_k_num(params : &mut bloom_filter_params) -> Result<(), String> {
    let value : i32 = unsafe { externals::bf_ideal_k_num(params as *mut bloom_filter_params) };

    if value < 0 {
        return util::strerror(value);
    }

    return Ok(());
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong};
    use super::{bloom_bloomfilter, bloom_filter_params};
    use bitmap::bloom_bitmap;

    pub const MAGIC_HEADER : c_uint = 0xCB1005DD;

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
    use super::{bloom_bloomfilter, bloom_filter_params, create_bloom_filter};
    use filter;

    static BITMAP_FILE : &'static str = "/tmp/map.bmp";

    #[test]
    fn test() {
        let params : bloom_filter_params = filter::test::create_bloom_filter_params();
        let filter : bloom_bloomfilter = create_bloom_filter(&params, BITMAP_FILE);

        filter::test::test_filter(Box::new(filter),
            &[[true, false, false], [false, true, false], [false, false, true]],
            &[[true, false, false], [true, true, false], [true, true, true]]);
    }
}

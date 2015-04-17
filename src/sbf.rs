#![allow(unstable)]

extern crate libc;
use self::libc::{c_char, c_int, c_ulong, c_void, malloc, size_t};
use std::{mem, ffi, ptr};
use bitmap::bloom_bitmap;
use bloom::bloom_bloomfilter;
use filter::IBloomFilter;
use util;

#[repr(C, packed)]
pub struct bloom_sbf_params {
    initial_capacity      : u64,
    fp_probability        : f64,
    scale_size            : u32,
    probability_reduction : f64
}

impl bloom_sbf_params {
    pub fn new(initial_capacity : u64, fp_probability : f64, scale_size : u32, probability_reduction : f64) -> Self {
        return bloom_sbf_params { initial_capacity: initial_capacity, fp_probability: fp_probability, scale_size: scale_size, probability_reduction: probability_reduction };
    }
}

pub type bloom_sbf_callback = extern "C" fn (*mut c_void, c_ulong, *mut bloom_bitmap) -> c_int;

extern "C" fn default_callback(input : *mut c_void, bytes : c_ulong, out : *mut bloom_bitmap) -> c_int {
    return 0;
}
    
#[repr(C)]
pub struct bloom_sbf {
    params         : bloom_sbf_params,
    callback       : bloom_sbf_callback,
    callback_input : *mut c_void,
    num_filters    : u32,
    filters        : Vec<bloom_bloomfilter>,
    dirty_filters  : Vec<u8>,
    capacities     : Vec<u64>
}

impl bloom_sbf {
    pub fn with_callback(params         : bloom_sbf_params,
                         callback       : bloom_sbf_callback,
                         callback_input : *mut c_void,
                         num_filters    : u32,
                         filters        : Vec<bloom_bloomfilter>,
                         dirty_filters  : Vec<u8>,
                         capacities     : Vec<u64>) -> Self {
        return bloom_sbf {
            params: params,
            callback: callback,
            callback_input: callback_input,
            num_filters: num_filters,
            filters: filters,
            dirty_filters: dirty_filters,
            capacities: capacities
        };
    }

    pub fn new(params        : bloom_sbf_params,
               num_filters   : u32,
               filters       : Vec<bloom_bloomfilter>,
               dirty_filters : Vec<u8>,
               capacities    : Vec<u64>) -> Self {
        return bloom_sbf::with_callback(params, default_callback, ptr::null_mut(), num_filters, filters, dirty_filters, capacities);
    }
    
    pub fn from_filters_with_callback(params         : bloom_sbf_params,
                                      callback       : bloom_sbf_callback,
                                      callback_input : *mut c_void,
                                      filters        : Vec<bloom_bloomfilter>) -> Result<Self, String> {
        let mut dirty_filters : Vec<u8> = Vec::new();
        let mut capacities : Vec<u64> = Vec::new();

        for filter in filters.iter() {
            dirty_filters.push(0);
            capacities.push(0);
        }

        let mut sbf : bloom_sbf = bloom_sbf::with_callback(params, callback, callback_input, filters.len() as u32, filters, dirty_filters, capacities);

        let value : i32 = unsafe { externals::sbf_from_filters(&mut sbf.params as *mut bloom_sbf_params, callback, callback_input, sbf.num_filters, sbf.filters.as_mut_slice() as *mut [bloom_bloomfilter], &mut sbf as *mut bloom_sbf) };

        if value < 0 {
            return util::strerror(value);
        }
        
        return Ok(sbf);
    }

    pub fn from_filters(params : bloom_sbf_params, filters : Vec<bloom_bloomfilter>) -> Result<Self, String> {
        return bloom_sbf::from_filters_with_callback(params, default_callback, ptr::null_mut(), filters);
    }

    pub fn total_capacity(&self) -> u64 {
        return unsafe { externals::sbf_total_capacity(self as *const bloom_sbf) };
    }

    pub fn total_byte_size(&self) -> u64 {
        return unsafe { externals::sbf_total_byte_size(self as *const bloom_sbf) };
    }
}

impl IBloomFilter<bool> for bloom_sbf {   
    fn add(&mut self, key : String) -> Result<bool, String> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_add(self as *mut bloom_sbf, key.as_ptr()) };

        if result < 0 {
            return util::strerror(result);
        }

        return Ok(result > 0);
    }

    fn contains(&self, key : &String) -> Result<bool, String> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_contains(self as *const bloom_sbf, key.as_ptr()) };

        if result < 0 {
            return util::strerror(result);
        }

        return Ok(result > 0);
    }

    fn size(&self) -> u64 { 
        return unsafe { externals::sbf_size(self as *const bloom_sbf) };
    }

    fn flush(&mut self) -> Result<(), String> {
        let value : u32 = unsafe { externals::sbf_flush(self as *mut bloom_sbf) };

        if value < 0 {
            return util::strerror(value);
        }

        return Ok(());
    }
}

impl Drop for bloom_sbf {
    fn drop(&mut self) {
        unsafe { externals::sbf_close(self as *mut bloom_sbf) };
    }
}

mod externals {
    use super::libc::{c_char, c_double, c_int, c_uint, c_ulong, c_void};
    use bloom::bloom_bloomfilter;
    use super::{bloom_sbf, bloom_sbf_params, bloom_sbf_callback};
    use bitmap::bloom_bitmap;

    #[link(name = "bloom")]
    extern {
        pub fn sbf_from_filters(params : *mut bloom_sbf_params, cb : bloom_sbf_callback, cb_in : *mut c_void, num_filters : u32, filters : *mut [bloom_bloomfilter], sbf : *mut bloom_sbf) -> c_int;

        pub fn sbf_add(filter : *mut bloom_sbf, key : *const c_char) -> c_int;

        pub fn sbf_contains(filter : *const bloom_sbf, key : *const c_char) -> c_int;

        pub fn sbf_size(filter : *const bloom_sbf) -> c_ulong;

        pub fn sbf_flush(filter : *mut bloom_sbf) -> c_int;

        pub fn sbf_close(filter : *mut bloom_sbf) -> c_int;

        pub fn sbf_total_capacity(filter : *const bloom_sbf) -> c_ulong;

        pub fn sbf_total_byte_size(filter : *const bloom_sbf) -> c_ulong;
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use super::{bloom_sbf, bloom_sbf_params};
    use bloom::{bloom_bloomfilter, bloom_filter_params, create_bloom_filter};
    use filter;

    #[test]
    fn test() {
        let sbf_params : bloom_sbf_params = bloom_sbf_params::new(100000, 0.0001, 4, 0.9);
        let bf_params : bloom_filter_params = filter::test::create_bloom_filter_params();

        let mut filters : Vec<bloom_bloomfilter> = Vec::new();

        for i in (0..3) {
            filters.push(create_bloom_filter(&bf_params, format!("/tmp/sbf-map{}.bmp", i).as_slice()));
        }

        let filter : bloom_sbf = bloom_sbf::from_filters(sbf_params, filters);

        filter::test::test_filter(Box::new(filter),
            &[[true, false, false], [false, true, false], [false, false, true]],
            &[[true, false, false], [true, true, false], [true, true, true]]);
    }
}

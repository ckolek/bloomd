#![allow(unstable)]

extern crate libc;
use self::libc::{c_char, c_int, c_ulong, c_void, malloc, size_t};
use std::{mem, ffi, ptr};
use bitmap::bloom_bitmap;
use bloom::bloom_bloomfilter;
use filter::IBloomFilter;

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

#[repr(C)]
pub struct bloom_sbf<'a> {
    params         : bloom_sbf_params,
    callback       : bloom_sbf_callback,
    callback_input : *mut c_void,
    num_filters    : u32,
    filters        : Vec<bloom_bloomfilter<'a>>,
    dirty_filters  : Vec<u8>,
    capacities     : Vec<u64>
}

pub type bloom_sbf_callback = Option<extern "C" fn (c_void, c_ulong, bloom_bitmap) -> c_int>;
    
impl<'a> bloom_sbf<'a> {
    pub fn new(params        : bloom_sbf_params,
               cb            : bloom_sbf_callback,
               cb_in         : *mut c_void,
               num_filters   : u32,
               filters       : Vec<bloom_bloomfilter<'a>>,
               dirty_filters : Vec<u8>,
               capacities    : Vec<u64>) -> Self {
        return bloom_sbf {
            params: params,
            callback: cb,
            callback_input: cb_in,
            num_filters: num_filters,
            filters: filters,
            dirty_filters: dirty_filters,
            capacities: capacities
        };
    }
    
    pub fn from_filters(params  : bloom_sbf_params,
                        cb          : bloom_sbf_callback,
                        cb_in       : *mut c_void,
                        filters     : Vec<bloom_bloomfilter<'a>>) -> Self {
        let mut sbf : bloom_sbf = bloom_sbf::new(params, cb, cb_in, filters.len() as u32, filters, Vec::new(), Vec::new());
        unsafe { 
            externals::sbf_from_filters(&mut sbf.params as *mut bloom_sbf_params, cb, cb_in, sbf.num_filters, &mut sbf.filters[0] as *mut bloom_bloomfilter, &mut sbf as *mut bloom_sbf)
        };
        
        return sbf;
    }
    
    pub fn total_capacity(&self) -> u64 {
        return unsafe { externals::sbf_total_capacity(self as *const bloom_sbf) };
    }

    pub fn total_byte_size(&self) -> u64 {
        return unsafe { externals::sbf_total_byte_size(self as *const bloom_sbf) };
    }
}

impl<'a> IBloomFilter<bool> for bloom_sbf<'a> {   
    fn add(&mut self, key : String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_add(self as *mut bloom_sbf, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    fn contains(&self, key : &String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_contains(self as *const bloom_sbf, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    fn size(&self) -> u64 { 
        return unsafe { externals::sbf_size(self as *const bloom_sbf) };
    }

    fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::sbf_flush(self as *mut bloom_sbf) } < 0 {
            return Err(());
        } else {
            return Ok(());
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for bloom_sbf<'a> {
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
        pub fn sbf_from_filters(params : *mut bloom_sbf_params, cb : bloom_sbf_callback, cb_in : *mut c_void, num_filters : u32, filters : *mut bloom_bloomfilter, sbf : *mut bloom_sbf) -> c_int;

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
    use bloom::{bloom_bloomfilter, bloom_filter_params};
    use filter;

    #[test]
    fn test() {
        let sbf_params : bloom_sbf_params = bloom_sbf_params::new(100000, 0.0001, 4, 0.9);
        let bf_params : bloom_filter_params = filter::test::create_bloom_filter_params();

        let mut filters : Vec<bloom_bloomfilter> = Vec::new();

        for i in (0..3) {
            filters.push(filter::test::create_bloom_filter(&bf_params, format!("sbf-map{}.bmp", i).as_slice()));
        }

        let filter : bloom_sbf = bloom_sbf::from_filters(sbf_params, None, ptr::null_mut(), filters);

        filter::test::test_filter(Box::new(filter),
            &[[true, false, false], [false, true, false], [false, false, true]],
            &[[true, false, false], [true, true, false], [true, true, true]]);
    }
}

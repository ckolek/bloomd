#![allow(unstable)]

extern crate libc;

#[repr(C, packed)]
pub struct bloom_sbf_params {
    initial_capacityv : u64,
    fp_probability : f64,
    scale_size : u32,
    probability_reduction : f64
};

#[repr(C)]
pub struct bloom_sbf {
    params : bloom_sbf_params,
    callback : bloom_sbf_callback,
    callback_input : c_void,
    num_filters : u32,
    filters [bloom_bloomfilter],
    dirty_filters : [u8],
    capacities : [u64]
};

pub type bloom_sbf_callback = extern fn (c_void, c_ulong, bloom_bitmap) -> c_int;
    
impl bloom_sbf {
    pub fn sbf_new(params : bloom_sbf_params,
                   cb : bloom_sbf_callback,
                   cb_in : c_void,
                   num_filters : u32,
                   filters : [bloom_bloomfilter],
                   dirty_filters : [u8],
                   capacities : [u64]) {
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
    
    pub fn sbf_from_filters(params : bloom_sbf_params,
                            cb : bloom_sbf_callback,
                            cb_in : c_void,
                            num_filters : u32,
                            filters : [bloom_bloomfilter]) -> Self {
        let sbf : bloom_sbf = sbf_new(params, cb, cb_in, num_filters, filters, &[0; 0], &[0; 0]);
        unsafe { 
            externals::sbf_from_filters(params, cb, cb_in, num_filters,
                                        filters, &mut sbf as *mut bloom_sbf)
        };
        
        return sbf;
    }
    
    pub fn add(&mut self, key : String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_add(self as *mut bloom_sbf, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    pub fn contains(&self, key : &String) -> Result<bool, ()> {
        let key : ffi::CString = ffi::CString::from_slice(key.as_slice().as_bytes());

        let result : i32 = unsafe { externals::sbf_contains(self as *const bloom_sbf, key.as_ptr()) };
        if result < 0 {
            return Err(());
        } else {
            return Ok(result > 0);
        }
    }

    pub fn size(&self) -> u64 
        return unsafe { externals::sbf_size(self as *const bloom_sbf) };
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::sbf_flush(self as *mut bloom_sbf) } < 0 {
            return Err(());
        } else {
            return Ok(());
        }
    }
    
    pub fn sbf_total_capacity(&self) -> u64 {
        return unsafe { externals::sbf_total_capacity(self as *const bloom_sbf) };
    }

    pub fn sbf_total_byte_size(&self) -> u64 {
        return unsafe { externals::sbf_total_byte_size(self as *const bloom_sbf) };
    }
}

impl Drop for bloom_sbf {
    fn drop(&mut self) {
        unsafe { externals::sbf_close(self as *mut bloom_sbf) };
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong, c_double};
    use bloom::bloom_bloomfilter;
    use super::bloom_sbf;
    use bitmap::bloom_bitmap;

    #[link(name = "bloom")]
    extern {
        pub fn sbf_from_filters(params : bloom_sbf_params,
                                cb : bloom_sbf_callback,
                                cb_in : c_void,
                                num_filters : u32,
                                filters : [bloom_bloomfilter],
                                sbf : bloom_sbf) -> c_int;
        pub fn sbf_add(filter : *mut bloom_sbf, key : *const c_char) -> c_int;
        pub fn sbf_contains(filter : *const bloom_sbf, key : *const c_char) -> c_int;
        pub fn sbf_size(filter : *const bloom_sbf) -> c_ulong;
        pub fn sbf_flush(filter : *mut bloom_sbf) -> c_int;
        pub fn sbf_close(filter : *mut bloom_sbf) -> c_int;
        pub fn uint64_t sbf_total_capacity(filter : *mut bloom_sbf) -> c_ulong;
        pub fn uint64_t sbf_total_byte_size(filter : *mut bloom_sbf) -> c_ulong;
    }
}
#![allow(unstable)]

extern crate libc;

#[repr(C, packed)]
pub struct bloom_sbf_params {
    initial_capacityv : u64,
    fp_probability : double,
    scale_size : u32,
    probability_reduction : double
};

/*
#[repr(C)]
pub struct bloom_sbf(c_void) {
    params : bloom_sbf_params,
    bloom_sbf_callback callback,
    callback_input : *void,
    num_filters : u32,
    bloom_bloomfilter **filters,
    dirty_filters : *mut char,
    capacities : *u64
};
*/
#[repr(C)]
pub struct bloom_sbf(c_void);
    
impl bloom_sbf {
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

impl Drop for bloom_sbf {
    fn drop(&mut self) {
        unsafe { externals::sbf_close(self as *mut bloom_sbf) };
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong, c_double};
    use bloom::bloom_bloomfilter;
    use super::bloom_sbf
    use bitmap::bloom_bitmap;

    #[link(name = "bloom")]
    extern {
        pub fn sbf_add(filter : *mut bloom_sbf, key : *const c_char) -> c_int;
        pub fn sbf_contains(filter : *const bloom_sbf, key : *const c_char) -> c_int;
        pub fn sbf_size(filter : *const bloom_sbf) -> c_ulong;
        pub fn sbf_flush(filter : *mut bloom_sbf) -> c_int;
        pub fn sbf_close(filter : *mut bloom_sbf) -> c_int;
        pub fn uint64_t sbf_total_capacity(filter : *mut bloom_sbf) -> c_ulong;
        pub fn uint64_t sbf_total_byte_size(filter : *mut bloom_sbf) -> c_ulong;
    }
}
extern crate libc;

use std::{ffi};
use bloom::bloom_bloomfilter;

#[repr(C)]
pub struct bloom_lbf {
    num_filters : u32,
    filters : Vec<bloom_bloomfilter>
}

impl bloom_lbf {
    fn add(&mut self, key : String) -> i32 {
        let mut index : i32 = 0;

        for ref mut filter in self.filters.iter_mut() {
            index += 1;

            if !filter.contains(&key).unwrap() {
                filter.add(key).unwrap();

                return index;
            }
        }

        return 0;
    }

    fn contains(&self, key : &String) -> i32 {
        let mut index : i32 = 0;

        for ref filter in self.filters.iter() {
            if !filter.contains(key).unwrap() {
                break;
            }
            
            index += 1;
        }

        return index;
    }

    fn size(&self) -> u64 {
        return self.filters[0].size();
    }

    fn flush(&mut self) -> i32 {
        let mut value : i32 = 0;

        for ref mut filter in self.filters.iter_mut() {
            if filter.flush().is_err() {
                value = -1;
            }
        }

        return value;
    }
}

#[unsafe_destructor]
impl Drop for bloom_lbf {
    fn drop(&mut self) {
        for ref mut filter in self.filters.iter() {
            drop(filter);
        }
    }
}

pub extern "C" fn lbf_add(lbf : *mut bloom_lbf, key : &str) -> i32 {
    return unsafe { (*lbf).add(String::from_str(key)) };
}

pub extern "C" fn lbf_contains(lbf : *mut bloom_lbf, key : &str) -> i32 {
    return unsafe { (*lbf).contains(&String::from_str(key)) };
}

pub extern "C" fn lbf_size(lbf : *mut bloom_lbf) -> u64 {
    return unsafe { (*lbf).size() };
}

pub extern "C" fn lbf_flush(lbf : *mut bloom_lbf) -> i32 {
    return unsafe { (*lbf).flush() };
}

pub extern "C" fn lbf_close(lbf : *mut bloom_lbf) -> i32 {
    unsafe { drop(&mut *lbf) };
    return 1;
}

extern crate libc;

use filter::BloomFilter;
use bloom::bloom_bloomfilter;

#[repr(C)]
pub struct bloom_lbf<'a> {
    num_filters : u32,
    filters : Vec<bloom_bloomfilter<'a>>
}

impl<'a> bloom_lbf<'a> {
    pub fn new(filters : Vec<bloom_bloomfilter<'a>>) -> Self {
        return bloom_lbf { num_filters: filters.len() as u32, filters: filters };
    }
}

impl<'a> BloomFilter<u32> for bloom_lbf<'a> {
    fn add(&mut self, key : String) -> Result<u32, ()> {
        let mut index : u32 = 0;

        for ref mut filter in self.filters.iter_mut() {
            index += 1;

            match filter.contains(&key) {
                Ok(in_filter) => {
                    if !in_filter {
                        return match filter.add(key) {
                            Ok(_) => Ok(index),
                            Err(_) => Err(())
                        }
                    }
                },
                Err(_) => return Err(())
            }
        }

        return Ok(0);
    }

    fn contains(&self, key : &String) -> Result<u32, ()> {
        let mut index : u32 = 0;
        let mut last_filter_index : u32 = 0;
        
        for ref filter in self.filters.iter() {
            index += 1;
            match filter.contains(key) {
                Ok(in_filter) => { 
                    if in_filter { 
                        last_filter_index = index; 
                    } 
                    else {
                        break;
                    }
                },
                Err(e) => return Err(e)
            }
        }

        return Ok(last_filter_index);
    }

    fn size(&self) -> u64 {
        return self.filters[0].size();
    }

    fn flush(&mut self) -> Result<(), ()> {
        let mut result : Result<(), ()> = Ok(());

        for ref mut filter in self.filters.iter_mut() {
            if filter.flush().is_err() {
                result = Err(());
            }
        }

        return result;
    }
}

#[unsafe_destructor]
impl<'a> Drop for bloom_lbf<'a> {
    fn drop(&mut self) {
        for ref mut filter in self.filters.iter() {
            drop(filter);
        }
    }
}

#[no_mangle]
pub extern "C" fn lbf_add(lbf : *mut bloom_lbf, key : &str) -> i32 {
    return unsafe {
        match (*lbf).add(String::from_str(key)) {
            Ok(value) => value as i32,
            Err(_) => -1
        }
    };
}

#[no_mangle]
pub extern "C" fn lbf_contains(lbf : *mut bloom_lbf, key : &str) -> i32 {
    return unsafe {
        match (*lbf).contains(&String::from_str(key)) {
            Ok(value) => value as i32,
            Err(_) => -1
        }
    };
}

#[no_mangle]
pub extern "C" fn lbf_size(lbf : *mut bloom_lbf) -> u64 {
    return unsafe { (*lbf).size() };
}

#[no_mangle]
pub extern "C" fn lbf_flush(lbf : *mut bloom_lbf) -> i32 {
    return unsafe {
        match (*lbf).flush() {
            Ok(_) => 0,
            Err(_) => -1
        }
    };
}

#[no_mangle]
pub extern "C" fn lbf_close(lbf : *mut bloom_lbf) -> i32 {
    unsafe { drop(&mut *lbf) };
    return 1;
}

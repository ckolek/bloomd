extern crate libc;

use filter::IBloomFilter;
use bloom::{bloom_bloomfilter, bloom_filter_params};

#[repr(C)]
pub struct bloom_lbf {
    pub params      : bloom_filter_params,
    pub name        : String,
    pub num_filters : u32,
    filters         : Vec<bloom_bloomfilter>
}

impl bloom_lbf {    
    pub fn new(params  : bloom_filter_params,
               name    : String,
               filters : Vec<bloom_bloomfilter>) -> Self {
        return bloom_lbf {
            params: params,
            name: name,
            num_filters: filters.len() as u32,
            filters: Vec::new()
        };
    }

    pub fn add_filter(&mut self, filter : bloom_bloomfilter) {
        self.filters.push(filter);
        self.num_filters += 1;
    }
}

impl IBloomFilter<u32> for bloom_lbf {
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
        
        for ref filter in self.filters.iter() {
            match filter.contains(key) {
                Ok(in_filter) => { if !in_filter { break } },
                Err(_) => return Err(())
            }

            index += 1;
        }

        return Ok(index);
    }

    fn size(&self) -> u64 {
        if !self.filters.is_empty() {
            return self.filters[0].size();
        } else {
            return 0;
        }
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

impl Drop for bloom_lbf {
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

#[cfg(test)]
mod tests {
    use filter;
    use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter};
    use lbf::bloom_lbf;

    #[test]
    fn test() {
        let params : bloom_filter_params = filter::test::create_bloom_filter_params();
        let lbf : bloom_lbf = bloom_lbf::new(params, &String::from_str("test"), Vec::new());

        filter::test::test_filter(Box::new(lbf),
            &[[1, 0, 0], [2, 1, 0], [3, 2, 1]],
            &[[1, 0, 0], [2, 1, 0], [3, 2, 1]]);
    }

}

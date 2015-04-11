extern crate libc;

use filter::BloomFilter;
use bloom::{bloom_bloomfilter, bloom_filter_params, create_bloom_filter};

#[repr(C)]
pub struct bloom_lbf<'a> {
    filter_params : bloom_filter_params,
    filter_name : String,
    num_filters : u32,
    filters : Vec<bloom_bloomfilter<'a>>
}

impl<'a> bloom_lbf<'a> {    
    pub fn new(filter_params : bloom_filter_params,
               filter_name : &String,
               filters : Vec<bloom_bloomfilter<'a>>) -> Self {
        return bloom_lbf {
            filter_params: filter_params,
            filter_name: filter_name.clone(),
            num_filters: filters.len() as u32,
            filters: Vec::new()
        };
    }
}

impl<'a> BloomFilter<u32> for bloom_lbf<'a> {
    fn add(&mut self, key : String) -> Result<u32, ()> {
        let mut index : u32 = 0;

        // add to the first filter that doesn't have the key, then return how many
        // layers we went down
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
        
        // If the key was in all the filters, make a new one
        let mut filter : bloom_bloomfilter = create_bloom_filter(&self.filter_params, 
            format!("{}-map{}.bmp", self.filter_name.as_slice(), self.num_filters).as_slice());
        return match filter.add(key) {
            Ok(_) => {
                self.filters.push(filter);
                self.num_filters += 1;
                return Ok(self.num_filters);
            },
            Err(_) => Err(())
        };
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

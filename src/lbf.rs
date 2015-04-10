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

#[cfg(test)]
mod tests {
    use bloom::{bloom_filter_header, bloom_bloomfilter, bloom_filter_params, size_for_capacity_prob, ideal_k_num};
    use bitmap::{bitmap_mode, bloom_bitmap};
    use filter::BloomFilter;
    use super::bloom_lbf;
    
    fn create_bloom_filter(params : &bloom_filter_params, index : i32) -> bloom_bloomfilter {
        let mut map : bloom_bitmap = bloom_bitmap::from_filename(format!("map{}.bmp", index).as_slice(), 1000000, true, bitmap_mode::NEW_BITMAP).unwrap();
        return bloom_bloomfilter::new(map, params.k_num, true);
    }
    
    #[test]
    fn test() {
        let mut params : bloom_filter_params = bloom_filter_params::empty();
        params.capacity = 1000000;
        params.fp_probability = 0.001;

        size_for_capacity_prob(&mut params).unwrap();
        ideal_k_num(&mut params).unwrap();

        println!("bytes: {}, k_num: {}", params.bytes, params.k_num);

        let mut filters : Vec<bloom_bloomfilter> = Vec::new();

        let mut map : bloom_bitmap;
        let mut header : bloom_filter_header;
        let mut filter : bloom_bloomfilter;

        for i in (0..3) {
            filters.push(create_bloom_filter(&params, i));
        }

        let mut lbf : bloom_lbf = bloom_lbf::new(filters);

        let key1 : String = String::from_str("abc");
        let key2 : String = String::from_str("def");
        let key3 : String = String::from_str("ghi");

        // add first key
        assert!(lbf.add(key1.clone()).unwrap() == 1);
        assert!(lbf.size() == 1);
        assert!(lbf.contains(&key1).unwrap() == 1);
        assert!(lbf.contains(&key2).unwrap() == 0);
        assert!(lbf.contains(&key3).unwrap() == 0);

        // add second key
        assert!(lbf.add(key1.clone()).unwrap() == 2);
        assert!(lbf.add(key2.clone()).unwrap() == 1);
        assert!(lbf.size() == 2);
        assert!(lbf.contains(&key1).unwrap() == 2);
        assert!(lbf.contains(&key2).unwrap() == 1);
        assert!(lbf.contains(&key3).unwrap() == 0);

        // add third key
        assert!(lbf.add(key1.clone()).unwrap() == 3);
        assert!(lbf.add(key2.clone()).unwrap() == 2);
        assert!(lbf.add(key3.clone()).unwrap() == 1);
        assert!(lbf.size() == 3);
        assert!(lbf.contains(&key1).unwrap() == 3);
        assert!(lbf.contains(&key2).unwrap() == 2);
        assert!(lbf.contains(&key3).unwrap() == 1);

        assert!(lbf.size() == 3);

        lbf.flush().unwrap();
    }
}
#![allow(unstable)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

use bloom::{bloom_filter_params, bloom_filter_header, bloom_bloomfilter, size_for_capacity_prob, ideal_k_num};
use bitmap::{bitmap_mode, bloom_bitmap};

mod bloom;
mod bitmap;
mod lbf;

fn main() {
    let mut params : bloom_filter_params = bloom_filter_params::empty();
    params.capacity = 1000000;
    params.fp_probability = 0.001;

    size_for_capacity_prob(&mut params).unwrap();
    ideal_k_num(&mut params).unwrap();

    println!("bytes: {}, k_num: {}", params.bytes, params.k_num);

    let mut map : bloom_bitmap = bloom_bitmap::from_filename("map1.bmp", 1000000, true, bitmap_mode::NEW_BITMAP).unwrap();

    let header : bloom_filter_header = bloom_filter_header::new(0, params.k_num, 0);

    let mut filter : bloom_bloomfilter = bloom_bloomfilter::new(&header, &mut map, params.k_num, true);

    let key1 : String = String::from_str("abc");
    let key2 : String = String::from_str("def");
    let key3 : String = String::from_str("ghi");

    // add first key
    assert!(filter.add(key1.clone()).unwrap());
    assert!(filter.contains(&key1).unwrap());
    assert!(!filter.contains(&key2).unwrap());
    assert!(!filter.contains(&key3).unwrap());

    // add second key
    assert!(!filter.add(key1.clone()).unwrap());
    assert!(filter.add(key2.clone()).unwrap());
    assert!(filter.contains(&key1).unwrap());
    assert!(filter.contains(&key2).unwrap());
    assert!(!filter.contains(&key3).unwrap());

    // add third key
    assert!(!filter.add(key1.clone()).unwrap());
    assert!(!filter.add(key2.clone()).unwrap());
    assert!(filter.add(key3.clone()).unwrap());
    assert!(filter.contains(&key1).unwrap());
    assert!(filter.contains(&key2).unwrap());
    assert!(filter.contains(&key3).unwrap());

    assert!(filter.size() == 3);

    filter.flush().unwrap();
}

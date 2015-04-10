#![allow(unstable)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

use filter::BloomFilter;
use bloom::{bloom_filter_params, bloom_filter_header, bloom_bloomfilter, size_for_capacity_prob, ideal_k_num};
use bitmap::{bitmap_mode, bloom_bitmap};
use lbf::bloom_lbf;

mod filter;
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
    assert!(lbf.add(key1.clone()).unwrap() == 0);
    assert!(lbf.contains(&key1).unwrap() == 1);
    assert!(lbf.contains(&key2).unwrap() == 0);
    assert!(lbf.contains(&key3).unwrap() == 0);

    // add second key
    assert!(lbf.add(key1.clone()).unwrap() == 1);
    assert!(lbf.add(key2.clone()).unwrap() == 0);
    assert!(lbf.contains(&key1).unwrap() == 2);
    assert!(lbf.contains(&key2).unwrap() == 1);
    assert!(lbf.contains(&key3).unwrap() == 0);

    // add third key
    assert!(lbf.add(key1.clone()).unwrap() == 2);
    assert!(lbf.add(key2.clone()).unwrap() == 1);
    assert!(lbf.add(key3.clone()).unwrap() == 0);
    assert!(lbf.contains(&key1).unwrap() == 3);
    assert!(lbf.contains(&key2).unwrap() == 2);
    assert!(lbf.contains(&key3).unwrap() == 1);

    assert!(lbf.size() == 3);

    lbf.flush().unwrap();
}

fn create_bloom_filter(params : &bloom_filter_params, index : i32) -> bloom_bloomfilter {
    let mut map : bloom_bitmap = bloom_bitmap::from_filename(format!("map{}.bmp", index).as_slice(), 1000000, true, bitmap_mode::NEW_BITMAP).unwrap();

    return bloom_bloomfilter::new(map, params.k_num, true);
}

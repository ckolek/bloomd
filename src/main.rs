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

}
#![allow(unstable)]

extern crate libc;

use bitmap::bloom_bitmap;

#[repr(C, packed)]
pub struct bloom_filter_header {
    magic : u32,
    k_num : u32,
    count : u64,
    __buf : [u8; 496]
}

#[repr(C)]
pub struct bloom_filter {
    header      : bloom_filter_header,
    map         : bloom_bitmap,
    offset      : u64,
    bitmap_size : u64
}

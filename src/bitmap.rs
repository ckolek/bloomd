
extern crate libc;

use std::{ffi};
use std::ops::{BitAnd, BitOr, BitXor};

#[repr(C)]
#[derive(Copy)]
pub enum bitmap_mode {
    SHARED     = 1,
    PERSISTENT = 2,
    ANONYMOUS  = 4,
    NEW_BITMAP = 8
}

impl BitAnd for bitmap_mode {
    type Output = u32;

    fn bitand(self, _rhs : bitmap_mode) -> u32 {
        return self as u32 & _rhs as u32;
    }
}

impl BitOr for bitmap_mode {
    type Output = u32;

    fn bitor(self, _rhs : bitmap_mode) -> u32 {
        return self as u32 | _rhs as u32;
    }
}

impl BitXor for bitmap_mode {
    type Output = u32;

    fn bitxor(self, _rhs : bitmap_mode) -> u32 {
        return self as u32 ^ _rhs as u32;
    }
}

#[repr(C)]
pub struct bloom_bitmap {
    mode        : u32,
    fileno      : i32,
    size        : u64,
    mmap        : Vec<i8>,
    dirty_pages : Vec<i8>
}

impl bloom_bitmap {
    // Returns a new bitmap 
    fn new(mode : u32, fileno : i32, size : u64) -> Self {
        return bloom_bitmap { mode: mode, fileno: fileno, size: size, mmap: Vec::with_capacity(size as usize), dirty_pages: Vec::with_capacity(size as usize) };
    }

    // Returns the bitmap from an opened file
    pub fn from_file(fileno : i32, len : u64, mode : u32) -> Result<Self, ()> {
        let mut map : bloom_bitmap = bloom_bitmap::new(mode, fileno, len);

        if unsafe { externals::bitmap_from_file(fileno, len, mode, &mut map as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(map);
    }

    // Opens the file with the name given and loads the bitmap in it
    pub fn from_filename(filename : &str, len : u64, create : bool, mode : u32) -> Result<Self, ()> {
        let mut map : bloom_bitmap  = bloom_bitmap::new(mode, 0, len);

        let filename : ffi::CString = ffi::CString::from_slice(filename.as_bytes());

        if unsafe { externals::bitmap_from_filename(filename.as_ptr(), len, create as i32, mode, &mut map as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(map);
    }
    
    // Flushes the changes to the bitmap to the disk
    pub fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::bitmap_flush(self as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(());
    }
}

// Frees the C memory if the Rust object is dropped
#[unsafe_destructor]
impl Drop for bloom_bitmap {
    fn drop(&mut self) {
        unsafe { externals::bitmap_close(self as *mut bloom_bitmap) };
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_ulong};
    use super::{bloom_bitmap};

    #[link(name = "bloom")]
    extern {
        pub fn bitmap_from_file(fileno : c_int, len : c_ulong, mode : u32, map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_from_filename(filename : *const c_char, len : c_ulong, create : c_int, mode : u32, map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_flush(map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_close(map : *mut bloom_bitmap) -> c_int; 
    }
}

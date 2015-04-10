
extern crate libc;

use std::{ffi};

#[repr(C)]
#[derive(Copy)]
pub enum bitmap_mode {
    SHARED     = 1,
    PERSISTENT = 2,
    ANONYMOUS  = 3,
    NEW_BITMAP = 4
}

#[repr(C)]
pub struct bloom_bitmap<'a> {
    mode        : bitmap_mode,
    fileno      : i32,
    size        : u64,
    mmap        : &'a [u8],
    dirty_pages : &'a [u8]
}

impl<'a> bloom_bitmap<'a> {
    pub fn new(mode : bitmap_mode, fileno : i32, size : u64, mmap : &'a [u8], dirty_pages : &'a [u8]) -> Self {
        return bloom_bitmap { mode: mode, fileno: fileno, size: size, mmap: mmap, dirty_pages: dirty_pages };
    }

    pub fn from_file(fileno : i32, len : u64, mode : bitmap_mode) -> Self {
        let mut map : bloom_bitmap<'a> = bloom_bitmap::new(mode, fileno, len, &[0; 0], &[0; 0]);

        unsafe {
            externals::bitmap_from_file(fileno, len, mode, &mut map as *mut bloom_bitmap);
        }

        return map;
    }

    pub fn from_filename(filename : &str, len : u64, create : bool, mode : bitmap_mode) -> Self {
        let mut map : bloom_bitmap<'a> = bloom_bitmap::new(mode, 0, len, &[0; 0], &[0; 0]);

        let filename : ffi::CString = ffi::CString::from_slice(filename.as_bytes());

        unsafe {            
            externals::bitmap_from_filename(filename.as_ptr(), len, create as i32, mode, &mut map as *mut bloom_bitmap);
        }

        return map;
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::bitmap_flush(self as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(());
    }
}

#[unsafe_destructor]
impl<'a> Drop for bloom_bitmap<'a> {
    fn drop(&mut self) {
        unsafe { externals::bitmap_close(self as *mut bloom_bitmap) };
    }
}

mod externals {
    use super::libc::{c_char, c_int, c_ulong};
    use super::{bitmap_mode, bloom_bitmap};

    #[link(name = "bloom")]
    extern {
        pub fn bitmap_from_file(fileno : c_int, len : c_ulong, mode : bitmap_mode, map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_from_filename(filename : *const c_char, len : c_ulong, create : c_int, mode : bitmap_mode, map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_flush(map : *mut bloom_bitmap) -> c_int;

        pub fn bitmap_close(map : *mut bloom_bitmap) -> c_int; 
    }
}

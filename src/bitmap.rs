
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
pub struct bloom_bitmap {
    mode        : bitmap_mode,
    fileno      : i32,
    size        : u64,
    mmap        : Vec<i8>,
    dirty_pages : Vec<i8>
}

impl bloom_bitmap {
    fn new(mode : bitmap_mode, fileno : i32, size : u64) -> Self {
        return bloom_bitmap { mode: mode, fileno: fileno, size: size, mmap: Vec::with_capacity(size as usize), dirty_pages: Vec::with_capacity(size as usize) };
    }

    pub fn from_file(fileno : i32, len : u64, mode : bitmap_mode) -> Result<Self, ()> {
        let mut map : bloom_bitmap = bloom_bitmap::new(mode, fileno, len);

        if unsafe { externals::bitmap_from_file(fileno, len, mode, &mut map as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(map);
    }

    pub fn from_filename(filename : &str, len : u64, create : bool, mode : bitmap_mode) -> Result<Self, ()> {
        let mut map : bloom_bitmap  = bloom_bitmap::new(mode, 0, len);

        let filename : ffi::CString = ffi::CString::from_slice(filename.as_bytes());

        if unsafe { externals::bitmap_from_filename(filename.as_ptr(), len, create as i32, mode, &mut map as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(map);
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        if unsafe { externals::bitmap_flush(self as *mut bloom_bitmap) } < 0 {
            return Err(());
        }

        return Ok(());
    }
}

#[unsafe_destructor]
impl Drop for bloom_bitmap {
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

#[cfg(test)]
mod tests {
    use super::{bitmap_mode, bloom_bitmap};

    #[test]
    fn test() {
    }
}

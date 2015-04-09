
extern crate libc;

#[repr(C)]
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
    mmap        : Vec<u8>,
    dirty_pages : Vec<u8>
}

mod externals {
    use super::libc::{c_char, c_int, c_uint, c_ulong};
    use super::{bitmap_mode, bloom_bitmap};

    #[link(name = "bloom")]
    extern {
        fn bitmap_from_file(fileno : c_int, len : c_ulong, mode : bitmap_mode, map : *mut bloom_bitmap) -> c_int;

        fn bitmap_from_filename(filename : *mut c_char, len : c_ulong, create : c_int, mode : bitmap_mode, map : *mut bloom_bitmap) -> c_int;

        fn bitmap_flush(map : *mut bloom_bitmap) -> c_int;

        fn bitmap_close(map : *mut bloom_bitmap) -> c_int; 
    }
}

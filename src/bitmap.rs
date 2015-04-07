
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


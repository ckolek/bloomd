
use bloom::bloom_filter;

#[repr(C)]
pub struct lbf {
    num_filters : u32,
    filters : Vec<bloom_filter>
}

impl lbf {
    fn add(&mut self, key : &str) -> Result<usize, ()> {
        return Err(());
    }

    fn contains(&self, key : &str) -> Option<usize> {
        return None;
    }

    fn size(&self) -> usize {
        return 0;
    }

    fn flush(&mut self) -> Result<(), ()> {
        return Err(());
    }

    fn close(&mut self) -> Result<(), ()> {
        return Err(());
    }
}

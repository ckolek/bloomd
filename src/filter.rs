
pub trait BloomFilter<T> {
    fn add(&mut self, key : String) -> Result<T, ()>;
    fn contains(&self, key : &String) -> Result<T, ()>;
    fn size(&self) -> u64;
    fn flush(&mut self) -> Result<(), ()>;
}

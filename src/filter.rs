
pub trait IBloomFilter<T> {
    fn add(&mut self, key : String) -> Result<T, ()>;
    fn contains(&self, key : &String) -> Result<T, ()>;
    fn size(&self) -> u64;
    fn flush(&mut self) -> Result<(), ()>;
}

pub mod test {
    use super::IBloomFilter;
    use bloom;
    use bloom::bloom_filter_params;

    static FILTER_CAPACITY : u64 = 1000000;
    static FILTER_FP_PROBABILITY : f64 = 0.001;
    
    pub fn create_bloom_filter_params() -> bloom_filter_params {
        return bloom::create_bloom_filter_params(FILTER_CAPACITY, FILTER_FP_PROBABILITY);
    }

    pub fn test_filter<T : Eq>(mut filter : Box<IBloomFilter<T>>, add_values : &[[T; 3]], contains_values : &[[T; 3]]) {
        let key1 : String = String::from_str("abc");
        let key2 : String = String::from_str("def");
        let key3 : String = String::from_str("ghi");

        // add first key
        assert!(filter.add(key1.clone()).unwrap() == add_values[0][0]);

        println!("here");

        assert!(filter.size() == 1);

        println!("here");

        assert!(filter.contains(&key1).unwrap() == contains_values[0][0]);
        assert!(filter.contains(&key2).unwrap() == contains_values[0][1]);
        assert!(filter.contains(&key3).unwrap() == contains_values[0][2]);

        // add second key
        assert!(filter.add(key1.clone()).unwrap() == add_values[1][0]);
        assert!(filter.add(key2.clone()).unwrap() == add_values[1][1]);

        assert!(filter.size() == 2);

        assert!(filter.contains(&key1).unwrap() == contains_values[1][0]);
        assert!(filter.contains(&key2).unwrap() == contains_values[1][1]);
        assert!(filter.contains(&key3).unwrap() == contains_values[1][2]);

        // add third key
        assert!(filter.add(key1.clone()).unwrap() == add_values[2][0]);
        assert!(filter.add(key2.clone()).unwrap() == add_values[2][1]);
        assert!(filter.add(key3.clone()).unwrap() == add_values[2][2]);

        assert!(filter.size() == 3);

        assert!(filter.contains(&key1).unwrap() == contains_values[2][0]);
        assert!(filter.contains(&key2).unwrap() == contains_values[2][1]);
        assert!(filter.contains(&key3).unwrap() == contains_values[2][2]);

        filter.flush().unwrap();
    }
}


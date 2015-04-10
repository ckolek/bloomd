
pub trait BloomFilter<T> {
    fn add(&mut self, key : String) -> Result<T, ()>;
    fn contains(&self, key : &String) -> Result<T, ()>;
    fn size(&self) -> u64;
    fn flush(&mut self) -> Result<(), ()>;
}

pub fn test_filter<T : Eq>(mut filter : Box<BloomFilter<T>>, add_values : &[[T; 3]], contains_values : &[[T; 3]]) {
    let key1 : String = String::from_str("abc");
    let key2 : String = String::from_str("def");
    let key3 : String = String::from_str("ghi");

    // add first key
    assert!(filter.add(key1.clone()).unwrap() == add_values[0][0]);

    assert!(filter.size() == 1);

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

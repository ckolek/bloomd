use config::{BloomConfig, BloomFilterConfig};
use bitmap::{bitmap_mode, bloom_bitmap};
use bloom::{bloom_filter_params, bloom_bloomfilter, size_for_capacity_prob, ideal_k_num};
use lbf::bloom_lbf;
use std::sync::{Mutex, RwLock};
use std::collections::hash_map::{HashMap, Iter};

pub struct FilterCounters {
    pub check_hits   : u64,
    pub check_misses : u64,
    pub set_hits     : u64,
    pub set_misses   : u64,
    pub page_ins     : u64,
    pub page_outs    : u64
}

impl FilterCounters {
    pub fn new() -> Self {
        return FilterCounters { check_hits: 0, check_misses: 0, set_hits: 0, set_misses: 0, page_ins: 0, page_outs: 0 };
    }
}

pub struct BloomFilter<'a> {
    pub config        : &'a BloomConfig,          // bloomd configuration
    pub filter_config : BloomFilterConfig,       // Filter-specific config
    pub lbf           : RwLock<bloom_lbf<'a>>, // Protects faulting in the filter
    pub counters      : FilterCounters             // Counters
}

impl<'a> BloomFilter<'a> {
    pub fn new(config : &'a BloomConfig, filter_config : BloomFilterConfig, lbf : RwLock<bloom_lbf<'a>>) -> BloomFilter<'a> {
        return BloomFilter {
            config        : config,
            filter_config : filter_config,
            lbf           : lbf,
            counters      : FilterCounters::new()
        };
    }
}

// Wrapper for dealing with RwLock
pub struct Filters<'a> {
    mutex   : Mutex<u8>,
    filters : HashMap<String, BloomFilter<'a>>
}

impl<'a> Filters<'a> {
    pub fn new() -> Self {
        return Filters {
            mutex: Mutex::new(0),
            filters: HashMap::new()
        };
    }

    pub fn contains_filter_named(&self, filter_name: &String) -> bool {
        self.mutex.lock().unwrap();

        return self.filters.contains_key(filter_name);
    }

    pub fn insert_filter(&mut self, filter_name : String, filter : BloomFilter<'a>) {
        self.mutex.lock().unwrap();

        self.filters.insert(filter_name, filter);
    }

    pub fn iter(&self) -> Iter<String, BloomFilter<'a>> {
        return self.filters.iter();
    }
}

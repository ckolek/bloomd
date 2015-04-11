use config::{BloomConfig, BloomFilterConfig};
use lbf::bloom_lbf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct FilterCounters {
    check_hits   : u64,
    check_misses : u64,
    set_hits     : u64,
    set_misses   : u64,
    page_ins     : u64,
    page_outs    : u64
}

impl FilterCounters {
    pub fn new() -> Self {
        return FilterCounters {
            check_hits   : 0,
            check_misses : 0,
            set_hits     : 0,
            set_misses   : 0,
            page_ins     : 0,
            page_outs    : 0
        };
    }
}

pub struct BloomFilter<'a, 'b> {
    config        : &'a BloomConfig,          // bloomd configuration
    filter_config : BloomFilterConfig,       // Filter-specific config
    full_path     : String,                    // Path to our data
    lbf_lock      : Mutex<bloom_lbf<'b>>, // Protects faulting in the filter
    counters      : FilterCounters             // Counters
}

impl<'a, 'b> BloomFilter<'a, 'b> {
    pub fn  new(config : &'a BloomConfig, filter_config : BloomFilterConfig, full_path : String, lbf : bloom_lbf<'b>) -> Self {
        return BloomFilter {
            config        : config,
            filter_config : filter_config,
            full_path     : full_path,
            lbf_lock      : Mutex::new(lbf),
            counters      : FilterCounters::new()
        };
    }
}

// Wrapper for dealing with RwLock
pub struct Filters<'a, 'b> {
    pub filters : HashMap<String, BloomFilter<'a, 'b>>
}

impl<'a, 'b> Filters<'a, 'b> {
    pub fn new() -> Self {
        return Filters {
            filters : HashMap::new()
        };
    }
}

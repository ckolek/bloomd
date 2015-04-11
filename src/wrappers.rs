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

pub struct BloomFilter<'a> {
    config        : &'a BloomConfig,          // bloomd configuration
    filter_config : BloomFilterConfig,       // Filter-specific config
    full_path     : String,                    // Path to our data
    lbf_lock      : Arc<Mutex<bloom_lbf<'a>>>, // Protects faulting in the filter
    counters      : FilterCounters             // Counters
}

// Wrapper for dealing with RwLock
pub struct Filters<'a> {
    pub filters : HashMap<String, BloomFilter<'a>>
}

impl<'a> Filters<'a> {
    pub fn new() -> Self {
        return Filters {
            filters : HashMap::new()
        };
    }
}

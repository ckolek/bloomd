use config::BloomFilterConfig;
use lbf::bloom_lbf;
use std::ops::{Deref, DerefMut};

pub struct BloomFilterCounters {
    pub check_hits   : u64,
    pub check_misses : u64,
    pub set_hits     : u64,
    pub set_misses   : u64,
    pub page_ins     : u64,
    pub page_outs    : u64
}

impl BloomFilterCounters {
    pub fn new() -> Self {
        return BloomFilterCounters { check_hits: 0, check_misses: 0, set_hits: 0, set_misses: 0, page_ins: 0, page_outs: 0 };
    }

    pub fn checks(&self) -> u64 {
        return self.check_hits + self.check_misses;
    }

    pub fn sets(&self) -> u64 {
        return self.set_hits + self.set_misses;
    }
}

pub struct BloomFilter {
    pub config   : BloomFilterConfig, // Filter-specific config
    pub lbf      : bloom_lbf,         // Layered bloom filter
    pub counters : BloomFilterCounters     // Counters
}

impl BloomFilter {
    pub fn new(config : BloomFilterConfig, lbf : bloom_lbf) -> BloomFilter {
        return BloomFilter {
            config   : config,
            lbf      : lbf,
            counters : BloomFilterCounters::new()
        };
    }
}

impl Deref for BloomFilter {
    type Target = bloom_lbf;

    fn deref<'a>(&'a self) -> &'a bloom_lbf {
        return &self.lbf;
    }
}

impl DerefMut for BloomFilter {
    fn deref_mut<'a>(&'a mut self) -> &'a mut bloom_lbf {
        return &mut self.lbf;
    }
}

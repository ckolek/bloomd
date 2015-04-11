use config::{bloom_config, bloom_filter_config};
use lbf::bloom_lbf;
use std::sync::{Arc, Mutex};

struct filter_counters {
    check_hits   : u64,
    check_misses : u64,
    set_hits     : u64,
    set_misses   : u64,
    page_ins     : u64,
    page_outs    : u64
}

struct bloom_filter<'a> {
    config        : &'a bloom_config,          // bloomd configuration
    filter_config : bloom_filter_config,       // Filter-specific config
    filter_name   : String,                    // The name of the filter
    full_path     : String,                    // Path to our data
    lbf_lock      : Arc<Mutex<bloom_lbf<'a>>>, // Protects faulting in the filter

    counters      : filter_counters        // Counters
}
use config::{BloomConfig, BloomFilterConfig};
use filter::IBloomFilter;
use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter_params};
use lbf::bloom_lbf;
use wrappers::{BloomFilter, Filters};
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::str::StrExt;

// ------------------------------------------------------------------
static MESSAGE_BAD_ARGS : &'static str = "Client Error: Bad arguments\r\n";
static MESSAGE_DONE     : &'static str = "Done\r\n";
static MESSAGE_EXISTS   : &'static str = "Exists\r\n";
static MESSAGE_NO_EXIST : &'static str = "Filter does not exist\r\n";
static MESSAGE_NO       : &'static str = "No";
// ------------------------------------------------------------------

// Sets many items in a filter at once
pub fn bulk<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }

    let mut results : Vec<u32> =  Vec::new();

    filters.use_filter(&filter_name, | filter | {
        // Write lock the filter so we don't interrupt anyone else reading
        let lbf : RwLockWriteGuard<bloom_lbf> = filter.lbf.write().unwrap();

        for key_name in args.iter() {
            results.push((*lbf).add(String::from_str(*key_name)).unwrap());
        }
    }).unwrap();

    return format!("{}\r\n", results.connect(" "));
}

// Checks if a key is in a filter
pub fn check<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    let result = filters.use_filter(&filter_name, | filter | {
        get_contains(filter.lbf.read().unwrap(), &key_name);
    }).unwrap();

    return format!("{}\r\n", result);
}

// Helper for check and multi, gets the output when given an lbf and a key to search for
pub fn get_contains(lbf : RwLockReadGuard<bloom_lbf>, key_name : &String) -> String {
    let result : u32 = lbf.contains(key_name).unwrap();
    if result == 0 {
        return String::from_str(MESSAGE_NO);
    }
    return format!("{}", result);
}

// Create a new filter
pub fn create<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() == 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name     : String = String::from_str(args.remove(0));
    let mut capacity    : u64 = config.initial_capacity;
    let mut probability : f64 = config.default_probability;
    let mut in_memory   : bool = config.in_memory;
    
    if filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_EXISTS);
    }
    
    // Check for changes to the capacity and probability
    for arg in args.iter() {
        if arg.starts_with("capacity=") {
            let mut pieces : Vec<&str> = arg.split_str("=").collect();
            match FromStr::from_str(pieces.pop().unwrap()) {
                Some(value) => { capacity = value },
                None => { }
            };
        }
        else if arg.starts_with("prob=") {
            let mut pieces : Vec<&str> = arg.split_str("=").collect();
            match FromStr::from_str(pieces.pop().unwrap()) {
                Some(value) => { probability = value },
                None => { }
            };
        }
        else if arg.starts_with("in_memory=") {
            let mut pieces : Vec<&str> = arg.split_str("=").collect();
            let value_opt : Option<u8> = FromStr::from_str(pieces.pop().unwrap());
            if value_opt.is_some() {
                in_memory = (value_opt.unwrap() > 0);
            }
        }
        else {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
    }
    // create the lbf and add it to the filters
    let params : bloom_filter_params = create_bloom_filter_params(capacity, probability);
    let filter_config : BloomFilterConfig = BloomFilterConfig::new(capacity, probability, in_memory, params.bytes, 0);
    let lbf : bloom_lbf = bloom_lbf::new(params, filter_name, Vec::new());

    filters.insert_filter(filter_name, BloomFilter::new(config, filter_config, RwLock::new(lbf)));

    return String::from_str(MESSAGE_DONE);
}

// Closes the filter (Unmaps from memory, but still accessible)
pub fn close<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("close {}\r\n", filter_name);
}

// Clears a filter from the lists (removes memory, left on disk)
pub fn clear<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("clear {}\r\n", filter_name);
}

// Drops a filter (deletes from disk)
pub fn drop<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("drop {}\r\n", filter_name);
}

// Gets info about filter
pub fn info<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args[0]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    // Acquire locks so no one changes the filter while we're reading
    let rlock = filters.lock.read();
    let filter : BloomFilter = filters.filters.get(filter_name).unwrap();
    let lbf : RwLockReadGuard<bloom_lbf> = filter.lbf_lock.read().unwrap();
    
    // Get info about filter
    let mut result : String = String::new();
    filters.use_filter(&filter_name, | filter | {
        // Write lock the filter so we don't interrupt anyone else reading
        let lbf : RwLockWriteGuard<bloom_lbf> = filter.lbf.write().unwrap();

        result.push_str("START\r\n");
        result.push_str(format!("capacity {}", filter.config.capacity).as_slice());
        result.push_str(format!("checks {}", filter.counters.check_hits + filter.counters.check_misses).as_slice());
        result.push_str(format!("check_hits {}", filter.counters.check_hits).as_slice());
        result.push_str(format!("check_misses {}", filter.counters.check_misses).as_slice());
        result.push_str(format!("page_ins {}", filter.counters.page_ins).as_slice());
        result.push_str(format!("page_outs {}", filter.counters.page_outs).as_slice());
        result.push_str(format!("probability {}", filter.config.probability).as_slice());
        result.push_str(format!("sets {}", filter.counters.sets).as_slice());
        result.push_str(format!("size {}", filter.config.size).as_slice());
        result.push_str(format!("storage {}", filter.config.bytes).as_slice());
        result.push_str("END\r\n");
    }).unwrap();
    
    return result;
}

// Lists all filters, or those matching a prefix
pub fn list<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    let mut prefix : &str = "";
    let mut result : String = String::new();
    
    if args.len() == 1 {
        prefix = args[0];
    }
    
    result.push_str("START\r\n");
    for (name, filter) in filters.iter() {
        if name.starts_with(prefix) {
            result.push_str(format!("{} {} {} {} {}\r\n", 
                                    name, 
                                    filter.filter_config.probability, 
                                    filter.filter_config.bytes, 
                                    filter.filter_config.capacity, 
                                    filter.filter_config.size).as_slice());
        }
    }
    result.push_str("END\r\n");
    
    return result;
}

// Checks if a list of keys are in a filter
pub fn multi<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    // Lock the filter so it doesn't change while we're reading
    let rlock = filters.lock.read();
    let filter : BloomFilter = filters.filters.get(filter_name).unwrap();
    let lbf : RwLockReadGuard<bloom_lbf> = filter.lbf_lock.read().unwrap();
    
    // Check each argument passed to us
    let results : Vec<&str> = Vec::new();
    for arg in args.iter() {
        results.push(get_contains(lbf, String::from_str(arg)).as_slice());
    }
    
    return format!("{}\r\n", results.connect(" "));
}

// Flushes all filters, or just a specified one
pub fn flush<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() > 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    if args.len() == 0 {
        return format!("flush all\r\n");
    }
    
    let filter_name  : String = String::from_str(args[0]);
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("flush {}\r\n", filter_name);
}

// Sets an item in a filter
pub fn set<'a>(config : &'a BloomConfig, filters : &Arc<Filters<'a>>, mut args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    if !filters.contains_filter_named(&filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    // Lock the filter so we don't interrupt anyone else reading
    let result = filters.use_filter(&filter_name, | filter | {
        // Write lock the filter so we don't interrupt anyone else reading
        let lbf : RwLockWriteGuard<bloom_lbf> = filter.lbf.write().unwrap();
        return (*lbf).add(key_name).unwrap();
    }).unwrap();
    
    return format!("{}\r\n", result);
}

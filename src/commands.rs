use config::{BloomConfig, BloomFilterConfig};
use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter_params};
use lbf::bloom_lbf;
use wrappers::{Filters, BloomFilter};
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::str::StrExt;

// ------------------------------------------------------------------
static MESSAGE_BAD_ARGS : &'static str = "Client Error: Bad arguments\r\n";
static MESSAGE_DONE     : &'static str = "Done\r\n";
static MESSAGE_EXISTS   : &'static str = "Exists\r\n";
static MESSAGE_NO_EXIST   : &'static str = "Filter does not exist\r\n";
// ------------------------------------------------------------------

// Sets many items in a filter at once
pub fn bulk(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("bulk {} {}\r\n", filter_name, args.connect(" "));
}

// Checks if a key is in a filter
pub fn check(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("check {} {}\r\n", filter_name, key_name);
}

// Create a new filter
pub fn create(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() == 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name     : String = String::from_str(args.remove(0));
    let mut capacity    : u64 = config.initial_capacity;
    let mut probability : f64 = config.default_probability;

    // If it already exists, don't create it again
    if filter_exists(filters, &filter_name) {
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
        else {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
    }
    
    // create the lbf and add it to the filters
    let params : bloom_filter_params = create_bloom_filter_params(capacity.clone(), probability.clone());
    let filter_params : BloomFilterConfig = BloomFilterConfig::new(capacity, probability);
    
    let lbf : bloom_lbf = bloom_lbf::new(params, &filter_name, Vec::new());
    let filter : BloomFilter = BloomFilter::new(config, 
                                                filter_params,
                                                format!("{}.{}", config.data_dir, filter_name),
                                                lbf);
    let write_filters : RwLockWriteGuard<Filters<'static>> = filters.write().unwrap();
    
    write_filters.filters.insert(filter_name, filter);
    return String::from_str(MESSAGE_DONE);
}

// Closes the filter (Unmaps from memory, but still accessible)
pub fn close(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("close {}\r\n", filter_name);
}

// Clears a filter from the lists (removes memory, left on disk)
pub fn clear(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("clear {}\r\n", filter_name);
}

// Drops a filter (deletes from disk)
pub fn drop(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
        if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("drop {}\r\n", filter_name);
}

// Gets info about filter
pub fn info(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args[0]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("info {}\r\n", filter_name);
}

// Lists all filters, or those matching a prefix
pub fn list(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    return String::from_str("list\r\n");
}

// Checks if a list of keys are in a filter
pub fn multi(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("multi {} {}\r\n", filter_name, args.connect(" "));
}

// Flushes all filters, or just a specified one
pub fn flush(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() > 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    if args.len() == 0 {
        return format!("flush all\r\n");
    }
    
    let filter_name  : String = String::from_str(args[0]);
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("flush {}\r\n", filter_name);
}

// Sets an item in a filter
pub fn set(config : &BloomConfig, filters : &Arc<RwLock<Filters<'static>>>, mut args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    if !filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_NO_EXIST);
    }
    
    return format!("set {} {}\r\n", filter_name, key_name);
}

pub fn filter_exists(filters : &Arc<RwLock<Filters<'static>>>, filter_name : &String) -> bool {
    let read_filters : RwLockReadGuard<Filters<'static>> = filters.read().unwrap();
    return read_filters.filters.contains_key(filter_name);
}

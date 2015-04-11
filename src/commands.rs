use wrappers::bloom_filter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

// ------------------------------------------------------------------
static MESSAGE_BAD_ARGS : &'static str = "Client Error: Bad arguments\r\n";
static MESSAGE_DONE     : &'static str = "Done\r\n";
// ------------------------------------------------------------------

// Sets many items in a filter at once
pub fn bulk(filters : &Arc<HashMap<String, bloom_filter>>, args : Vec<&str>) -> String {
    return String::from_str("in bulk\r\n");
}

// Checks if a key is in a filter
pub fn check(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in check\r\n");
}

// Create a new filter
pub fn create(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() == 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.remove(0));
    let mut capacity : u64 = 1000000;
    let mut prob     : f64 = 0.001;
    
    for arg in args.iter() {
        if arg.starts_with("capacity=") {
            capacity = match FromStr::from_str(arg.trim_left_matches("capacity=")) {
                Some(value) => { value },
                None => { 1000000 }
            };
        }
        else if arg.starts_with("prob=") {
            prob = match FromStr::from_str(arg.trim_left_matches("prob=")) {
                Some(value) => { value },
                None => { 0.001 }
            };
        }
    }
    
    return String::from_str(format!("create {} capacity={} prob={}", filter_name, capacity, prob));
}

// Closes the filter (Unmaps from memory, but still accessible)
pub fn close(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in close\r\n");
}

// Clears a filter from the lists (removes memory, left on disk)
pub fn clear(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in clear\r\n");
}

// Drops a filter (deletes from disk)
pub fn drop(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in drop\r\n");
}

// Gets info about filter
pub fn info(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in info\r\n");
}

// Lists all filters, or those matching a prefix
pub fn list(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in list\r\n");
}

// Checks if a list of keys are in a filter
pub fn multi(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in multi\r\n");
}

// Flushes all filters, or just a specified one
pub fn flush(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in flush\r\n");
}

// Sets an item in a filter
pub fn set(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    return String::from_str("in set\r\n");
}

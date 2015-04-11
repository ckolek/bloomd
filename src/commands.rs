use wrappers::bloom_filter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

// ------------------------------------------------------------------
static MESSAGE_BAD_ARGS : &'static str = "Client Error: Bad arguments\r\n";
static MESSAGE_DONE     : &'static str = "Done\r\n";
static MESSAGE_EXISTS   : &'static str = "Exists\r\n";
// ------------------------------------------------------------------

// Sets many items in a filter at once
pub fn bulk(filters : &Arc<HashMap<String, bloom_filter>>, args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    return String::from_str("bulk {} {}\r\n", filter_name, args.connect(" "));
}

// Checks if a key is in a filter
pub fn check(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    let key_name  : String = String::from_str(args.get(1));
    
    return format!("check {} {}\r\n", filter_name, key_name);
}

// Create a new filter
pub fn create(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() == 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name  : String = String::from_str(args.remove(0));
    let mut capacity : u64 = 1000000;
    let mut prob     : f64 = 0.001;
    
    if filter_exists(filters, &filter_name) {
        return String::from_str(MESSAGE_EXISTS);
    }
    
    for arg in args.iter() {
        if arg.starts_with("capacity=") {
            capacity = match FromStr::from_str::<u64>(arg.trim_left_matches("capacity=")) {
                Some(value) => { value },
                None => { 1000000 }
            };
        }
        else if arg.starts_with("prob=") {
            prob = match FromStr::from_str::<f64>(arg.trim_left_matches("prob=")) {
                Some(value) => { value },
                None => { 0.001 }
            };
        }
        else {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
    }
    
    return format!("create {} capacity={} prob={}\r\n", filter_name, capacity, prob);
}

// Closes the filter (Unmaps from memory, but still accessible)
pub fn close(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    
    return format!("close {}\r\n", filter_name);
}

// Clears a filter from the lists (removes memory, left on disk)
pub fn clear(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    
    return format!("clear {}\r\n", filter_name);
}

// Drops a filter (deletes from disk)
pub fn drop(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
        if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    
    return format!("drop {}\r\n", filter_name);
}

// Gets info about filter
pub fn info(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.get(0));
    
    return format!("info {}\r\n", filter_name);
}

// Lists all filters, or those matching a prefix
pub fn list(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    return String::from_str("list\r\n");
}

// Checks if a list of keys are in a filter
pub fn multi(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    return String::from_str("multi {} {}\r\n", filter_name, args.connect(" "));
}

// Flushes all filters, or just a specified one
pub fn flush(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() > 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    if args.len() == 0 {
        return format!("flush all\r\n");
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    return format!("flush {}\r\n", filter_name);
}

// Sets an item in a filter
pub fn set(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args.get(0));
    let key_name  : String = String::from_str(args.get(1));
    
    return format!("set {} {}\r\n", filter_name, key_name);
}

pub fn filter_exists(filters : &Arc<RwLock<HashMap<String, bloom_filter>>>, filter_name : &String) {
    let read_filters : RwLockReadGuard<HashMap<String, bloom_filter>> = filters.read().unwrap();
    return read_filters.contains_key(filter_name);
}

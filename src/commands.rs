use wrappers::{bloom_filter, Filters};
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
pub fn bulk<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    return format!("bulk {} {}\r\n", filter_name, args.connect(" "));
}

// Checks if a key is in a filter
pub fn check<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    return format!("check {} {}\r\n", filter_name, key_name);
}

// Create a new filter
pub fn create<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
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
        else {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
    }
    
    return format!("create {} capacity={} prob={}\r\n", filter_name, capacity, prob);
}

// Closes the filter (Unmaps from memory, but still accessible)
pub fn close<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    return format!("close {}\r\n", filter_name);
}

// Clears a filter from the lists (removes memory, left on disk)
pub fn clear<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    return format!("clear {}\r\n", filter_name);
}

// Drops a filter (deletes from disk)
pub fn drop<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
        if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    
    return format!("drop {}\r\n", filter_name);
}

// Gets info about filter
pub fn info<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args[0]);
    
    return format!("info {}\r\n", filter_name);
}

// Lists all filters, or those matching a prefix
pub fn list<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 0 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    return String::from_str("list\r\n");
}

// Checks if a list of keys are in a filter
pub fn multi<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() <= 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    // Get the arguments
    let filter_name : String = String::from_str(args.remove(0));
    
    return format!("multi {} {}\r\n", filter_name, args.connect(" "));
}

// Flushes all filters, or just a specified one
pub fn flush<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() > 1 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    if args.len() == 0 {
        return format!("flush all\r\n");
    }
    
    let filter_name  : String = String::from_str(args[0]);
    return format!("flush {}\r\n", filter_name);
}

// Sets an item in a filter
pub fn set<'a>(filters : &Arc<RwLock<Filters<'a>>>, args : Vec<&str>) -> String {
    if args.len() != 2 {
        return String::from_str(MESSAGE_BAD_ARGS);
    }
    
    let filter_name  : String = String::from_str(args[0]);
    let key_name  : String = String::from_str(args[1]);
    
    return format!("set {} {}\r\n", filter_name, key_name);
}

pub fn filter_exists<'a>(filters : &Arc<RwLock<Filters<'a>>>, filter_name : &str) -> bool {
    let read_filters : RwLockReadGuard<Filters<'a>> = filters.read().unwrap();
    return read_filters.filters.contains_key(&String::from_str(filter_name));
}

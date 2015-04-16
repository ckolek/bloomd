#![allow(unstable)]
#![allow(dead_code)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]

use config::{BloomConfig, BloomFilterConfig};
use filter::IBloomFilter;
use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter_params, create_bloom_filter};
use lbf::bloom_lbf;
use wrappers::BloomFilter;
use std::os;
use std::io;
use std::io::{fs, TcpListener, Listener, Acceptor, Stream, BufferedStream};
use std::io::fs::PathExtensions;
use std::path::Path;
use std::thread::Thread;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::str::FromStr;

mod bitmap;
mod bloom;
mod config;
mod filter;
mod inifile;
mod lbf;
mod wrappers;

// constants -------------------------------------------------------------------
const MESSAGE_START           : &'static str = "START\r\n";
const MESSAGE_END             : &'static str = "END";
const MESSAGE_DONE            : &'static str = "Done";
const MESSAGE_EXISTS          : &'static str = "Exists";
const MESSAGE_NO_EXIST        : &'static str = "Filter does not exist";
const MESSAGE_YES             : &'static str = "Yes";
const MESSAGE_NO              : &'static str = "No";
const MESSAGE_NOT_IMPLEMENTED : &'static str = "Client Error: Command not supported";
const MESSAGE_BAD_ARGS        : &'static str = "Client Error: Bad arguments";
const COMMAND_BULK_AB         : &'static str = "b";
const COMMAND_BULK            : &'static str = "bulk";
const COMMAND_CHECK_AB        : &'static str = "c";
const COMMAND_CHECK           : &'static str = "check";
const COMMAND_CREATE          : &'static str = "create";
const COMMAND_CLOSE           : &'static str = "close";
const COMMAND_CLEAR           : &'static str = "clear";
const COMMAND_DROP            : &'static str = "drop";
const COMMAND_INFO            : &'static str = "info";
const COMMAND_LIST            : &'static str = "list";
const COMMAND_MULTI_AB        : &'static str = "m";
const COMMAND_MULTI           : &'static str = "multi";
const COMMAND_FLUSH           : &'static str = "flush";
const COMMAND_SET_AB          : &'static str = "s";
const COMMAND_SET             : &'static str = "set";
// -----------------------------------------------------------------------------

// represents a bloom filter server
struct BloomServer {
    pub config  : BloomConfig,
    filters     : RwLock<HashMap<String, RwLock<BloomFilter>>>
}

impl BloomServer {
    // create a new BloomServer with the given configuration
    fn new(config : BloomConfig) -> Self {
        return BloomServer { config: config, filters: RwLock::new(HashMap::new()) };
    }
    
    // handle a client connection
    fn handle_client<S : Stream>(&self, stream: S) {
        let mut buf_stream : BufferedStream<S> = BufferedStream::new(stream);

        loop {
            // try to read input from client
            let line : String = match buf_stream.read_line() {
                Ok(results) => results,
                Err(_) => {
                    break;
                }
            }; 

            // clean up input
            let chars_to_trim: &[char] = &[' ', '\n', '\r'];
            let trim_line : &str = line.as_slice().trim_matches(chars_to_trim);

            // respond to input
            let response : String = self.interpret_request(trim_line);
            buf_stream.write_str(response.as_slice()).unwrap();
            buf_stream.write_str("\r\n").unwrap();

            buf_stream.flush().unwrap();
        };
    }

    // interpret a client request
    fn interpret_request(&self, input : &str) -> String {
        let mut args : Vec<&str> = input.split(|&:c : char| c.is_whitespace())
                                         .filter(|&s| s.len() > 0).collect();
        if args.is_empty() {
            return String::from_str(MESSAGE_NOT_IMPLEMENTED);
        }

        // get main command
        let command : &str = args.remove(0);

        // process command arguments and return response 
        return match command {
            COMMAND_BULK     => { self.process_bulk  (args) },
            COMMAND_BULK_AB  => { self.process_bulk  (args) },
            COMMAND_CHECK    => { self.process_check (args) },
            COMMAND_CHECK_AB => { self.process_check (args) },
            COMMAND_CREATE   => { self.process_create(args) },
            COMMAND_CLOSE    => { self.process_close (args) },
            COMMAND_CLEAR    => { self.process_clear (args) },
            COMMAND_DROP     => { self.process_drop  (args) },
            COMMAND_INFO     => { self.process_info  (args) },
            COMMAND_LIST     => { self.process_list  (args) },
            COMMAND_MULTI    => { self.process_multi (args) },
            COMMAND_MULTI_AB => { self.process_multi (args) },
            COMMAND_FLUSH    => { self.process_flush (args) },
            COMMAND_SET      => { self.process_set   (args) },
            COMMAND_SET_AB   => { self.process_set   (args) },
            _ => { String::from_str(MESSAGE_NOT_IMPLEMENTED) },
        }
    }

    // obtains a read lock on the filter HashMap and passes the reference to the given function
    // returns the result of calling the given function
    fn use_filters<T, F : Fn(&HashMap<String, RwLock<BloomFilter>>) -> T>(&self, user : F) -> T {
        return user(&*self.filters.read().unwrap());
    }

    // obtains a write lock on the filter HashMap and passes the mutable reference to the given function
    // returns the result of calling the given function
    fn use_filters_mut<T, F : Fn(&mut HashMap<String, RwLock<BloomFilter>>) -> T>(&self, user : F) -> T {
        return user(&mut *self.filters.write().unwrap());
    }

    // determines if there is a filter with the given name in the filter HashMap
    fn contains_filter_named(&self, filter_name : &String) -> bool {
        return self.use_filters(|filters| { filters.contains_key(filter_name) });
    }

    // obtains a read lock on the filter with the given name and passes the references to the given function
    // returns a Some with the result of calling the given function, or None if a filter with the given name does not exist
    fn use_filter<T, F : Fn(&BloomFilter) -> T>(&self, filter_name : &String, user : F) -> Option<T> {
        return self.use_filters(|filters| {
            return match filters.get(filter_name) {
                Some(filter) => { Some(user(&*filter.read().unwrap())) },
                None => None
            };
        });
    }

    // obtains a read lock on the filter with the given name and passes the mutable references to the given function
    // returns a Some with the result of calling the given function, or None if a filter with the given name does not exist
    fn use_filter_mut<T, F : Fn(&mut BloomFilter) -> T>(&self, filter_name : &String, user : F) -> Option<T> {
        return self.use_filters(|filters| {
            return match filters.get(filter_name) {
                Some(filter) => { Some(user(&mut *filter.write().unwrap())) },
                None => None
            };
        });
    }

    // process a 'bulk' command
    fn process_bulk(&self, args : Vec<&str>) -> String {
        if args.len() < 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        return self.use_filter_mut(&filter_name, |filter| {
            let mut result : String = String::new();

            for arg in args[1..].iter() {
                if !result.is_empty() {
                    result.push_str(" ");
                }

                let key : String = String::from_str(*arg);
                let value : u32 = self.set(filter, key);

                result.push_str(format!("{}", value).as_slice());
            }

            return result;
        }).unwrap();
    }

    // process a 'check' command
    fn process_check(&self, args : Vec<&str>) -> String {
        if args.len() != 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        return self.use_filter_mut(&filter_name, |filter| {
            let key : String = String::from_str(args[1]);
            let value : u32 = self.check(filter, key);

            return format!("{}", value);
        }).unwrap();
    }

    // process a 'create' command
    fn process_create(&self, args : Vec<&str>) -> String {
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_EXISTS);
        }
        
        let mut capacity    : u64  = self.config.initial_capacity;
        let mut probability : f64  = self.config.default_probability;
        let mut in_memory   : bool = self.config.in_memory;

        // Check for manual parameters
        for arg in args[1..].iter() {
            if arg.starts_with("capacity=") {
                let pieces : Vec<&str> = arg.split_str("=").collect();
                let value_opt : Option<u64> = FromStr::from_str(pieces[1]);
                if value_opt.is_some() {
                    capacity = value_opt.unwrap();
                }
            } else if arg.starts_with("prob=") {
                let pieces : Vec<&str> = arg.split_str("=").collect();
                let value_opt : Option<f64> = FromStr::from_str(pieces[1]);
                if value_opt.is_some() {
                    probability = value_opt.unwrap();
                }
            } else if arg.starts_with("in_memory=") {
                let pieces : Vec<&str> = arg.split_str("=").collect();
                let value_opt : Option<u8> = FromStr::from_str(pieces[1]);
                if value_opt.is_some() {
                    in_memory = value_opt.unwrap() > 0;
                }
            } else {
                return String::from_str(MESSAGE_BAD_ARGS);
            }
        }
        
        self.use_filters_mut(|filters| {
            let params : bloom_filter_params = create_bloom_filter_params(capacity, probability);
            let filter_config : BloomFilterConfig = BloomFilterConfig::new(capacity, probability, in_memory, params.bytes, 0);
            let lbf : bloom_lbf = bloom_lbf::new(params, filter_name.clone(), Vec::new());
            let bloom_filter : BloomFilter = BloomFilter::new(filter_config, lbf);

            filters.insert(filter_name.clone(), RwLock::new(bloom_filter))
        });

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'close' command
    fn process_close(&self, args : Vec<&str>) -> String {
        return String::new();
    }

    // process a 'clear' command
    fn process_clear(&self, args : Vec<&str>) -> String {
        return String::new();
    }

    // process a 'drop' command
    fn process_drop(&self, args : Vec<&str>) -> String {
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        self.use_filters_mut(|filters| {
            filters.remove(&filter_name);
        });

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'flush' command
    fn process_flush(&self, args : Vec<&str>) -> String {
        if args.len() > 1 {
            return String::from_str(MESSAGE_BAD_ARGS);
        } else if args.len() == 1 {
            let filter_name : String = String::from_str(args[0]);
            
            if !self.contains_filter_named(&filter_name) {
                return String::from_str(MESSAGE_NO_EXIST);
            }

            self.use_filter_mut(&filter_name, |filter| {
                filter.lbf.flush().unwrap();
            }).unwrap();
        } else {
            self.use_filters_mut(|filters| {
                for filter in filters.values() {
                    (*filter.write().unwrap()).flush().unwrap();
                }
            });
        }        

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'info' command
    fn process_info(&self, args : Vec<&str>) -> String {
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        return self.use_filter(&filter_name, |filter| {
            let mut result : String = String::new();
            result.push_str(MESSAGE_START);
            result.push_str(format!("capacity {}\r\nchecks {}\r\ncheck_hits {}\r\ncheck_misses {}\r\npage_ins {}\r\npage_outs {}\r\nprobability {}\r\nsets {}\r\nset_hits {}\r\nset_misses {}\r\nsize {}\r\nstorage {}\r\n",
                           filter.config.capacity,
                           filter.counters.checks(),
                           filter.counters.check_hits,
                           filter.counters.check_misses,
                           filter.counters.page_ins,
                           filter.counters.page_outs,
                           filter.config.probability,
                           filter.counters.sets(),
                           filter.counters.set_hits,
                           filter.counters.set_misses,
                           filter.config.size,
                           filter.config.bytes).as_slice());
            result.push_str(MESSAGE_END);

            return result;
        }).unwrap();
    }

    // process a 'list' command
    fn process_list(&self, args : Vec<&str>) -> String {
        if args.len() > 1 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        let mut prefix : &str;
        if !args.is_empty() {
            prefix = args[0];
        } else {
            prefix = "";
        }


        return self.use_filters(|filters| {
            let mut result : String = String::new();
            result.push_str(MESSAGE_START);

            for (name, filter_lock) in filters.iter() {
                if name.starts_with(prefix) {
                    let ref filter = *filter_lock.read().unwrap();

                    result.push_str(format!("{} {} {} {} {}\r\n", 
                                            name, 
                                            filter.config.probability, 
                                            filter.config.bytes, 
                                            filter.config.capacity, 
                                            filter.config.size).as_slice());
                }
            }

            result.push_str(MESSAGE_END);

            return result;
        });
    }

    // process a 'multi' command
    fn process_multi(&self, args : Vec<&str>) -> String {
        if args.len() < 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        return self.use_filter_mut(&filter_name, |filter| {
            let mut result : String = String::new();

            for arg in args[1..].iter() {
                if !result.is_empty() {
                    result.push_str(" ");
                }

                let key : String = String::from_str(*arg);
                let value : u32 = self.check(filter, key);

                result.push_str(format!("{}", value).as_slice());
            }

            return result;
        }).unwrap();
    }

    // process a 'set' command
    fn process_set(&self, args : Vec<&str>) -> String {
        if args.len() != 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        let filter_name : String = String::from_str(args[0]);
        
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        return self.use_filter_mut(&filter_name, |filter| {
            let key : String = String::from_str(args[1]);
            let value : u32 = self.set(filter, key);

            return format!("{}", value);
        }).unwrap();
    }

    // do a check for the given key in the given BloomFilter and return the corresponding value
    fn check(&self, filter : &mut BloomFilter, key : String) -> u32 {
        let ref lbf : bloom_lbf = filter.lbf;
        let value : u32 = lbf.contains(&key).unwrap();

        if value > 0 {
            filter.counters.check_hits += 1;
        } else {
            filter.counters.check_misses += 1;
        }

        return value;
    }

    // do a set for the given key in the given BloomFilter, creating new bloom filters if necessary, and return the corresponding value
    fn set(&self, filter : &mut BloomFilter, key : String) -> u32 {
        let ref mut lbf : bloom_lbf = filter.lbf;
        let value : u32 = lbf.contains(&key).unwrap();

        if value == lbf.num_filters {
            let bloom_filter : bloom_bloomfilter = create_bloom_filter(&lbf.params, format!("{}/{}-{}.bmp", &self.config.data_dir, &lbf.name, value).as_slice());

            lbf.add_filter(bloom_filter);
        }

        if value > 0 {
            filter.counters.set_hits += 1;
        } else {
            filter.counters.set_misses += 1;
        }

        let value = lbf.add(key).unwrap();

        filter.config.size = lbf.size();

        return value;
    }
}

unsafe impl Send for BloomServer { }

fn main() {
    // get command line arguments
    let args = os::args();
    let mut args = args.iter();
    args.next();

    let mut config_filename : Option<&str> = None;

    // read command line arguments
    loop {
        match args.next() {
            Some(arg) => {
                match arg.as_slice() {
                    // config filename
                    "-f" => {
                        match args.next() {
                            Some(value) => config_filename = Some(value.as_slice()),
                            None => panic!("missing value for flag \"-f\"")
                        }
                    },
                    _ => panic!("invalid argument: {}", arg)
                };
            },
            None => { break }
        };
    }

    // create config
    let config : BloomConfig = match config_filename {
        Some(filename) => BloomConfig::from_filename(filename),
        None => BloomConfig::default()
    };

    // create server
    let server : BloomServer = BloomServer::new(config);

    // start server
    start(server);
}

fn start(server: BloomServer) {
    // make sure data_dir exists and is accessible
    let data_dir : Path = Path::new(server.config.data_dir.clone());
    if !data_dir.exists() {
        fs::mkdir(&data_dir, io::USER_RWX).unwrap();
    } else if !data_dir.is_dir() {
        panic!("Invalid data_dir: {} is not a directory", data_dir.as_str().unwrap());
    }

    // listen at <bind_host>:<tcp_port>
    let listener = TcpListener::bind(server.config.get_bind_address().as_slice()).unwrap();

    let mut acceptor = listener.listen().unwrap();

    // share state
    let server : Arc<BloomServer> = Arc::new(server);

    // handle connection
    for stream in acceptor.incoming() {
        match stream {
            Ok(stream) => {
                let server = server.clone();

                Thread::spawn(move || {
                    server.handle_client(stream);
                });
            },
            Err(e) => { println!("failed to connect to incoming client: {}", e) }
        };
    }
}

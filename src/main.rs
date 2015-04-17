#![allow(unstable)]
#![allow(dead_code)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]
#![allow(improper_ctypes)]
#![feature(unboxed_closures)]

use config::{BloomConfig, BloomFilterConfig};
use filter::IBloomFilter;
use bloom::{bloom_filter_params, create_bloom_filter_params};
use lbf::bloom_lbf;
use wrappers::BloomFilter;
use std::os;
use std::io;
use std::io::{fs, TcpListener, Listener, Acceptor, Stream, BufferedStream};
use std::io::fs::PathExtensions;
use std::io::timer::Timer;
use std::time::Duration;
use std::path::Path;
use std::thread::Thread;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;
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
const USAGE                   : &'static str = "bloomd [-f config_file]";

const FILTER_FOLDER_PREFIX    : &'static str = "filter.";

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
    // create a new BloomServer with the given configuration, reading in pre-existing filters
    fn new(config : BloomConfig) -> Self {
        return BloomServer { config: config, filters: RwLock::new(HashMap::new()) };
    }

    // read existing filters from disk
    fn read_in_filters(&self) {
        let paths = fs::readdir(&Path::new(self.config.data_dir.clone())).unwrap();

        // Check each directory in the data directory
        for path in paths.iter() {
            if path.is_dir() {
                let mut components = path.str_components().collect::<Vec<Option<&str>>>();
                let last_component = components.pop().unwrap().unwrap();
                
                // If it is a bloom filter, it should start with the prefix 'filter.' followed by the filter name
                if last_component.starts_with(FILTER_FOLDER_PREFIX) {
                    self.use_filters_mut(|filters| {
                        let filter_name : String = String::from_str(&last_component[FILTER_FOLDER_PREFIX.len()..]);
                        
                        // If it is a valid filter, add it to the filters list
                        match BloomFilter::from_directory(path, &filter_name, false) {
                            Ok(filter) => { filters.insert(filter_name, RwLock::new(filter)); return (); },
                            Err(_) => { println!("Could not read filter from directory: {}", path.display()) }
                        };
                    });
                }
            }
        }
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
        // split input into valid arguments
        let mut args : Vec<&str> = input.split(|&:c : char| c.is_whitespace())
                                         .filter(|&s| s.len() > 0).collect();

        // handle empty input
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

    // process a 'bulk' command (bulk <filter> <key> ...)
    // returns a response String
    fn process_bulk(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() < 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // form response
        return self.use_filter_mut(&filter_name, |filter| {
            let mut result : String = String::new();

            // iterate through remaining arguments
            for arg in args[1..].iter() {
                if !result.is_empty() {
                    result.push_str(" ");
                }

                // get key and corresponding 'set' value
                let key : String = String::from_str(*arg);
                let value : u32 = self.set(filter, key);

                result.push_str(format!("{}", value).as_slice());
            }

            return result;
        }).unwrap();
    }

    // process a 'check' command (check <filter> <key>)
    // returns a response String
    fn process_check(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() != 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // form response
        return self.use_filter_mut(&filter_name, |filter| {
            let key : String = String::from_str(args[1]);
            let value : u32 = self.check(filter, key);

            return format!("{}", value);
        }).unwrap();
    }

    // process a 'create' command (create <filter> [capacity=<capacity>] [probability=<probability>] [in_memory=<in_memory>])
    // returns a response String
    fn process_create(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter does not exist
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
        
        // form response
        self.use_filters_mut(|filters| {
            let mut directory : Path = Path::new(self.config.data_dir.clone());
            directory.push(format!("{}{}", FILTER_FOLDER_PREFIX, &filter_name).as_slice());

            let bloom_filter : BloomFilter;
            if directory.exists() {
                bloom_filter = BloomFilter::from_directory(&directory, &filter_name, true).unwrap();
            } else {
                fs::mkdir(&directory, io::USER_RWX).unwrap();

                let params : bloom_filter_params = create_bloom_filter_params(capacity, probability);
                let filter_config : BloomFilterConfig = BloomFilterConfig::new(filter_name.clone(), capacity, probability, params.k_num, in_memory, params.bytes);
                let lbf : bloom_lbf = bloom_lbf::new(params, filter_name.clone(), Vec::new());

                bloom_filter = BloomFilter::new(filter_config, lbf, directory);
            }

            filters.insert(filter_name.clone(), RwLock::new(bloom_filter))
        });

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'close' command (close <filter>)
    // returns a response String
    fn process_close(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // remove the filter
        self.use_filter_mut(&filter_name, |filter| {
            filter.unload_filter();
        });

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'clear' command (clear <filter>)
    // returns a response String
    fn process_clear(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // remove the filter
        self.use_filters_mut(|filters| {
            filters.remove(&filter_name);
        });

        return String::from_str(MESSAGE_DONE);
    }

    // returns a response String (drop <filter>)
    // process a 'drop' command
    fn process_drop(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // remove the filter
        self.use_filters_mut(|filters| {
            let filter : Option<RwLock<BloomFilter>> = filters.remove(&filter_name);
            filter.unwrap().write().unwrap().delete();
        });

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'flush' command (flush [<filter>])
    // returns a response String
    fn process_flush(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() > 1 {
            return String::from_str(MESSAGE_BAD_ARGS);
        // handle single filter flush
        } else if args.len() == 1 {
            // get filter name
            let filter_name : String = String::from_str(args[0]);
            
            // check that filter exists
            if !self.contains_filter_named(&filter_name) {
                return String::from_str(MESSAGE_NO_EXIST);
            }

            // flush the filter
            self.use_filter_mut(&filter_name, |filter| {
                filter.flush().unwrap();
            }).unwrap();
        // handle all filters flush
        } else {
            // flush all filters
            self.use_filters_mut(|filters| {
                for filter in filters.values() {
                    (*filter.write().unwrap()).flush().unwrap();
                }
            });
        }        

        return String::from_str(MESSAGE_DONE);
    }

    // process a 'info' command (info <filter>)
    // returns a response String
    fn process_info(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.is_empty() {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // form response
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

    // process a 'list' command (list [<filter_prefix>])
    // returns a response String
    fn process_list(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() > 1 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter prefix
        let mut prefix : &str;
        if !args.is_empty() {
            prefix = args[0];
        } else {
            prefix = "";
        }

        // form response
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

    // process a 'multi' command (multi <filter> <key> ...)
    // returns a response String
    fn process_multi(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() < 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }

        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // form response
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

    // process a 'set' command (set <filter> <key>)
    // returns a response String
    fn process_set(&self, args : Vec<&str>) -> String {
        // handle invalid arguments
        if args.len() != 2 {
            return String::from_str(MESSAGE_BAD_ARGS);
        }
        
        // get filter name
        let filter_name : String = String::from_str(args[0]);
        
        // check that filter exists
        if !self.contains_filter_named(&filter_name) {
            return String::from_str(MESSAGE_NO_EXIST);
        }

        // form response
        return self.use_filter_mut(&filter_name, |filter| {
            let key : String = String::from_str(args[1]);
            let value : u32 = self.set(filter, key);

            return format!("{}", value);
        }).unwrap();
    }

    // do a check for the given key in the given BloomFilter and return the corresponding value
    fn check(&self, filter : &mut BloomFilter, key : String) -> u32 {
        filter.touch();

        let value : u32 = filter.contains(&key).unwrap();

        if value > 0 {
            filter.counters.check_hits += 1;
        } else {
            filter.counters.check_misses += 1;
        }

        return value;
    }

    // do a set for the given key in the given BloomFilter, creating new bloom filters if necessary, and return the corresponding value
    fn set(&self, filter : &mut BloomFilter, key : String) -> u32 {
        filter.touch();

        // Check and make sure that there is a layer that doesn't contain the key,
        // creating a new layer if necessary
        let value : u32 = filter.contains(&key).unwrap();
        if value == filter.num_filters {
            filter.add_filter(value);
        }
        
        // Increment the counters for the filter
        if value > 0 {
            filter.counters.set_hits += 1;
        } else {
            filter.counters.set_misses += 1;
        }

        // Add the key to the filter, and increment the size of the filter
        let value = filter.add(key).unwrap();
        let index : usize = (value - 1) as usize;

        filter.config.size = filter.size();
        filter.config.filter_sizes[index] = filter.get_filter_size(index);

        return value;
    }
}

unsafe impl Send for BloomServer { }

// represents a task that executes subtasks periodically
struct Worker {
    timer    : Timer,
    receiver : Receiver<()>,
    tasks    : Vec<Box<FnMut<(u64,), ()> + 'static>>
}

impl Worker {
    fn new(duration : Duration) -> Self {
        let mut timer : Timer = Timer::new().unwrap();
        let receiver : Receiver<()> = timer.periodic(duration);

        return Worker {
            timer: timer,
            receiver: receiver,
            tasks: Vec::new()
        };
    }

    fn add_task<T : FnMut<(u64,), ()> + 'static>(&mut self, task : T) {
        self.tasks.push(Box::new(task));
    }
}

impl FnMut<(), ()> for Worker {
    #[allow(unused_variables)]
    extern "rust-call" fn call_mut(&mut self, args : ()) {
        let mut count : u64 = 0;

        loop {
            // activate the tasks
            for task in self.tasks.iter_mut() {
                task(count,);
            }

            count += 1;

            // sleep for the rest of the minute
            self.receiver.recv().unwrap();
        }
    }
}

unsafe impl Send for Worker { }

// task for flushing filters
struct FlushTask {
    server     : Arc<BloomServer>,
    last_flush : u64
}

impl FlushTask {
    fn new(server : Arc<BloomServer>) -> Self {
        return FlushTask { server: server, last_flush: 0 };
    }
}

impl FnMut<(u64,), ()> for FlushTask {
    extern "rust-call" fn call_mut(&mut self, args : (u64,)) {
        let (time,) = args;

        if (time - self.last_flush) > self.server.config.flush_interval as u64 {
            self.server.use_filters_mut(|filters| {
                for filter in filters.values() {
                    (*filter.write().unwrap()).flush().unwrap();
                }
            });

            self.last_flush = time;
        }
    }
}

unsafe impl Send for FlushTask { }

// task for clearing cold filters
struct CloseTask {
    server : Arc<BloomServer>
}

impl CloseTask {
    fn new(server : Arc<BloomServer>) -> Self {
        return CloseTask { server: server };
    }
}

impl FnMut<(u64,), ()> for CloseTask {
    #[allow(unused_variables)]
    extern "rust-call" fn call_mut(&mut self, args : (u64,)) {
        self.server.use_filters_mut(|filters| {
            for filter_lock in filters.values() {
                let mut guard = filter_lock.write().unwrap();

                if (*guard).cold_index > self.server.config.cold_interval as u64 {
                    (*guard).unload_filter();
                }

                (*guard).cold_index += 1;
            }
        });
    }
}

unsafe impl Send for CloseTask { }

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
                            None => panic!("missing value for flag \"-f\"\r\n\r\n{}", USAGE)
                        }
                    },
                    _ => panic!("invalid argument: {}\r\n\r\n{}", arg, USAGE)
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
    } else {
        server.read_in_filters();
    }

    // listen at <bind_host>:<tcp_port>
    let listener = TcpListener::bind(server.config.get_bind_address().as_slice()).unwrap();

    let mut acceptor = listener.listen().unwrap();

    // share state
    let server : Arc<BloomServer> = Arc::new(server);

    // setup background tasks
    let flush_task : FlushTask = FlushTask::new(server.clone());
    let close_task : CloseTask = CloseTask::new(server.clone());

    let duration : Duration = Duration::minutes(1);

    // create worker threads
    if server.config.worker_threads <= 1 {
        let mut worker : Worker = Worker::new(duration);
        worker.add_task(flush_task);
        worker.add_task(close_task);

        Thread::spawn(worker);
    } else {
        let mut worker1 : Worker = Worker::new(duration);
        worker1.add_task(flush_task);

        let mut worker2 : Worker = Worker::new(duration);
        worker2.add_task(close_task);

        Thread::spawn(worker1);
        Thread::spawn(worker2);
    }

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

#[cfg(test)]
mod tests {
    use super::{BloomServer, MESSAGE_NO_EXIST, MESSAGE_EXISTS, MESSAGE_DONE, MESSAGE_BAD_ARGS, MESSAGE_NOT_IMPLEMENTED};
    use config::{BloomConfig};

    #[test]
    fn test_server () {
        // Start by making a default server instance
        let server : BloomServer = BloomServer::new(BloomConfig::default());
        
        // Test create, including when filter already exists
        test_command(&server, "create filter", MESSAGE_DONE);
        test_command(&server, "create filter", MESSAGE_EXISTS);
        
        // Test set (and check)
        test_command(&server, "check filter first", "0");
        test_command(&server, "set filter first", "1");
        test_command(&server, "c filter first", "1");
        test_command(&server, "s filter first", "2");
        test_command(&server, "c filter first", "2");
        test_command(&server, "s filter first", "3");
        test_command(&server, "c filter first", "3");
        
        // Test bad entries for set and check
        test_command(&server, "set filetr first", MESSAGE_NO_EXIST);
        test_command(&server, "check filetr first", MESSAGE_NO_EXIST);
        test_command(&server, "set filter first second", MESSAGE_BAD_ARGS);
        test_command(&server, "set filter first second", MESSAGE_BAD_ARGS);
        test_command(&server, "check filter", MESSAGE_BAD_ARGS);
        test_command(&server, "set filter", MESSAGE_BAD_ARGS);
        
        // Test multi and bulk
        test_command(&server, "multi filter first second third", "3 0 0");
        test_command(&server, "bulk filter first second third", "4 1 1");
        test_command(&server, "b filter first second third", "5 2 2");
        test_command(&server, "m filter first second third", "5 2 2");
        
        // Test bad entries for multi and bulk
        test_command(&server, "bulk filetr first second third", MESSAGE_NO_EXIST);
        test_command(&server, "multi filetr first second third", MESSAGE_NO_EXIST);
        test_command(&server, "check filter", MESSAGE_BAD_ARGS);
        test_command(&server, "set filter", MESSAGE_BAD_ARGS);
        
        // Test list
        test_command(&server, "list fake_prefix", "START\r\nEND");
        test_command(&server, "list", "START\r\nfilter 0.0001 239627 100000 3\r\nEND");
        
        // Test info
        let info_results : &str = "START\r\ncapacity 100000\r\nchecks 10\r\ncheck_hits 7\r\ncheck_misses 3\r\npage_ins 0\r\npage_outs 0\r\nprobability 0.0001\r\nsets 9\r\nset_hits 6\r\nset_misses 3\r\nsize 3\r\nstorage 239627\r\nEND";
        test_command(&server, "info", MESSAGE_BAD_ARGS);
        test_command(&server, "info filetr", MESSAGE_NO_EXIST);
        test_command(&server, "info filter", info_results);
        
        // Test nonexistent commands
        test_command(&server, "infor filter", MESSAGE_NOT_IMPLEMENTED);
        test_command(&server, "sette filter first", MESSAGE_NOT_IMPLEMENTED);
        
        // Clean up in case of persistence, and test drop
        test_command(&server, "drop", MESSAGE_BAD_ARGS);
        test_command(&server, "drop filter", MESSAGE_DONE);
        test_command(&server, "drop filter", MESSAGE_NO_EXIST);
    }
    
    fn test_command(server : &BloomServer, command : &str, result : &str) {
        assert_eq!(server.interpret_request(command).as_slice(),
                   result);
    }
}

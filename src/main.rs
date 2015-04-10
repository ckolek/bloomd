#![crate_type = "lib"]
#![crate_name = "bloomd"]

#![allow(unstable)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

mod filter;
mod bloom;
mod bitmap;
mod lbf;

#[cfg(not(test))]

static ADDRESS : &'static str = "127.0.0.1:8673";
static CONFIG_FILENAME : &'static str = "bloomd.config";

use config::bloom_config;
use wrappers::bloom_filter;
use std::collections::HashMap;

mod config;
mod wrappers;

struct BloomServer {
    config : bloom_config,
    filters : HashMap<String, bloom_filter>
}

impl BloomServer {
    fn new(config_filename : &str) -> Self {
        let config : bloom_config = // TODO: read config from file or create default

        return BloomServer { config: config, filters: HashMap::new() };
    }

    fn start(&mut self) {
        use std::io::{TcpListener,TcpStream};
        use std::io::{Listener,Acceptor};
        use std::thread::Thread;

        let listener = TcpListener::bind("127.0.0.1:8673");
        
        // bind the listener to the specified address
        let mut acceptor = listener.listen();

        // Accept incoming connections, with a new connection for each 
        for stream in acceptor.incoming() {
            match stream {
                Ok(stream) => {
                    Thread::spawn(move|| {
                        // connection made, now handle client
                        handle_client(stream);
                    });
                },
                Err(_) => { /* Could not connect */ }
            }
        }
    }

    fn handle_client<S : Stream>(&mut self, stream : S) {
        // TODO
    }
}

fn main() {
    let mut server : BloomServer = BloomServer::new(CONFIG_FILENAME);
    server.start();
}

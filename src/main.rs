/*
 *  Final Project
 *  Christopher Kolek, Jonathan Bower
 *
 *  Extends the bloomd server design to use lbfs
 */

#![allow(unstable)]
#![allow(unstable_features)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

use config::BloomConfig;
use wrappers::Filters;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::io::{Stream, BufferedStream};

mod filter;
mod bloom;
mod bitmap;
mod lbf;
mod commands;
mod config;
mod wrappers;
mod inifile;

// ------------------------------------------------------------------
const CONFIG_FILENAME         : &'static str = "bloomd.ini";
const MESSAGE_NOT_IMPLEMENTED : &'static str = "Client Error: Command not supported\r\n";
const MESSAGE_START           : &'static str = "START\r\n";
const MESSAGE_END             : &'static str = "END\r\n";
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
// ------------------------------------------------------------------

// Go ahead and start up the server
#[cfg(not(test))]
fn main() {
    use std::io::{TcpListener,TcpStream};
    use std::io::{Listener,Acceptor};
    use std::thread::Thread;

    let config : BloomConfig = BloomConfig::from_filename(CONFIG_FILENAME);
    
    //TODO: GET CONFIGS
    let filters : Arc<Filters> = Arc::new(Filters::new());
    let listener = TcpListener::bind(format!("{}:{}", config.bind_address, config.tcp_port).as_slice()).unwrap();
    
    // bind the listener to the specified address
    let mut acceptor = listener.listen().unwrap();

    // Accept incoming connections, with a new connection for each 
    for stream in acceptor.incoming() {
        let new_config = config.clone();
        let new_filters = filters.clone();
        
        match stream {
            Ok(stream) => {
                Thread::spawn(move|| {
                    // connection made, now handle client
                    handle_client(&new_config, &new_filters, stream);
                });
            },
            Err(_) => { /* Could not connect */ }
        }
    }
}
    
#[cfg(not(test))]
fn handle_client<S : Stream>(config: &BloomConfig, filters : &Arc<Filters<'static>>, stream : S) {
    let mut buf_stream : BufferedStream<S> = BufferedStream::new(stream);
    
    loop {
        let line : String = match buf_stream.read_line() {
            Ok(results) => results,
            Err(_) => {
	            break;
            }
        };	    
        
        let chars_to_trim: &[char] = &[' ', '\n', '\r'];
        let trim_line : &str = line.as_slice().trim_matches(chars_to_trim);

        let response : String = interpret_request(config, filters, trim_line);
        buf_stream.write_str(MESSAGE_START).unwrap();
        buf_stream.write_str(response.as_slice()).unwrap();
        buf_stream.write_str(MESSAGE_END).unwrap();

        // Need to flush, or else we won't write to the client
        buf_stream.flush().unwrap();
    };
}
    
// Find which command
fn interpret_request(config : &BloomConfig, filters : &Arc<Filters<'static>>, input : &str) -> String {
    let mut words : Vec<&str> = input.split(|&:c : char| c.is_whitespace())
                            .filter(|&s| s.len() > 0).collect();
    // If the line is empty, then exit
    if words.len() == 0 {
        return String::from_str(MESSAGE_NOT_IMPLEMENTED);
    }

    // Get the command to follow    
    let command : &str = words.remove(0);
    
    // Move to function for command, if command exists
    return match command {
        COMMAND_BULK     => { commands::bulk  (config, filters, words) },
        COMMAND_BULK_AB  => { commands::bulk  (config, filters, words) },
        COMMAND_CHECK    => { commands::check (config, filters, words) },
        COMMAND_CHECK_AB => { commands::check (config, filters, words) },
        COMMAND_CREATE   => { commands::create(config, filters, words) },
        COMMAND_CLOSE    => { commands::close (config, filters, words) },
        COMMAND_CLEAR    => { commands::clear (config, filters, words) },
        COMMAND_DROP     => { commands::drop  (config, filters, words) },
        COMMAND_INFO     => { commands::info  (config, filters, words) },
        COMMAND_LIST     => { commands::list  (config, filters, words) },
        COMMAND_MULTI    => { commands::multi (config, filters, words) },
        COMMAND_MULTI_AB => { commands::multi (config, filters, words) },
        COMMAND_FLUSH    => { commands::flush (config, filters, words) },
        COMMAND_SET      => { commands::set   (config, filters, words) },
        COMMAND_SET_AB   => { commands::set   (config, filters, words) },
        _ => { String::from_str(MESSAGE_NOT_IMPLEMENTED) },
    }
}


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

use config::bloom_config;
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
const BIND_ADDRESS            : &'static str = "127.0.0.1";
const BIND_TCP_PORT           : &'static str = "8673";
const CONFIG_FILENAME         : &'static str = "bloomd.config";
const MESSAGE_NOT_IMPLEMENTED : &'static str = "Client Error: Command not supported\r\n";
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
    
    //TODO: GET CONFIGS
    let filters_orig : Arc<RwLock<Filters>> = Arc::new(RwLock::new(Filters::new()));
    let listener = TcpListener::bind(format!("{}:{}", BIND_ADDRESS, BIND_TCP_PORT).as_slice()).unwrap();
    
    // bind the listener to the specified address
    let mut acceptor = listener.listen().unwrap();

    // Accept incoming connections, with a new connection for each 
    for stream in acceptor.incoming() {
        let filters_ref = filters_orig.clone();
        
        match stream {
            Ok(stream) => {
                Thread::spawn(move|| {
                    // connection made, now handle client
                    handle_client(&filters_ref, stream);
                });
            },
            Err(_) => { /* Could not connect */ }
        }
    }
}
    
#[cfg(not(test))]
fn handle_client<S : Stream>(filters : &Arc<RwLock<Filters<'static>>>, stream : S) {
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

        let response : String = interpret_request(filters, trim_line);
        buf_stream.write_str(response.as_slice()).unwrap();

        // Need to flush, or else we won't write to the client
        buf_stream.flush().unwrap();
    };
}
    
// Find which command
fn interpret_request(filters : &Arc<RwLock<Filters<'static>>>, input : &str) -> String {
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
        COMMAND_BULK     => { commands::bulk  (filters, words) },
        COMMAND_BULK_AB  => { commands::bulk  (filters, words) },
        COMMAND_CHECK    => { commands::check (filters, words) },
        COMMAND_CHECK_AB => { commands::check (filters, words) },
        COMMAND_CREATE   => { commands::create(filters, words) },
        COMMAND_CLOSE    => { commands::close (filters, words) },
        COMMAND_CLEAR    => { commands::clear (filters, words) },
        COMMAND_DROP     => { commands::drop  (filters, words) },
        COMMAND_INFO     => { commands::info  (filters, words) },
        COMMAND_LIST     => { commands::list  (filters, words) },
        COMMAND_MULTI    => { commands::multi (filters, words) },
        COMMAND_MULTI_AB => { commands::multi (filters, words) },
        COMMAND_FLUSH    => { commands::flush (filters, words) },
        COMMAND_SET      => { commands::set   (filters, words) },
        COMMAND_SET_AB   => { commands::set   (filters, words) },
        _ => { String::from_str(MESSAGE_NOT_IMPLEMENTED) },
    }
}


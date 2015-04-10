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

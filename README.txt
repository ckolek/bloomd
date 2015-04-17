Chris Kolek    <kolek.c@husky.neu.edu>
Jonathan Bower <jonathan.bower2010@gmail.com>

CS 4620 - Building Extensible Systems

Final Project: bloomd

/+  bloomd               - 'bloomd' Rust Cargo directory
 +-/+ csrc               - Original C source for bloomd
 |  +-/+ bloomd          - Contains C implementation of bloomd
 |  +-/+ libbloom        - Contains C implementation of bloom filters
 +-/+ deps               - Original C dependencies for bloomd
 +-/+ integ              - Original C integration for bloomd
 +-/+ src                - Rust source files directory
 |  +- bitmap.rs         - Interface for C bitmaps, using ffi
 |  +- bloom.rs          - Interface for C bloom filters, using ffi
 |  +- config.rs         - Declares bloom filter config structs
 |  +- filter.rs         - Declares bloom filter interface
 |  +- inifile.rs        - Init file reader, authored by Eliovir under open source license
 |  +- lbf.rs            - Implementation of layered bloom filters
 |  +- main.rs           - Runs the Rust server
 |  +- sbf.rs            - Interface for C scalable bloom filters, using ffi
 |  +- wrappers.rs       - Declares wrapper for bloom filters
 +-/+ tests              - Original C tests for bloomd, including counters
 +- bench                - used for benchmark testing for C
 +- build                - build file so Cargo builds both C and Rust bloomd servers
 +- Cargo.lock           - Cargo lock file
 +- Cargo.toml           - Cargo toml file
 +- README.txt           - This file explains all folder and files
 +- SConstruct           - Used by SCons to build C bloomd
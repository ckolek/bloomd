struct bloom_config {
    tcp_port              : i32,
    udp_port              : i32,
    bind_address          : String,
    data_dir              : String,
    log_level             : String,
    initial_capacity      : u64,
    default_probability   : f64,
    scale_size            : i32,
    probability_reduction : f64,
    flush_interval        : i32,
    cold_interval         : i32,
    in_memory             : bool,
    worker_threads        : i32,
    use_mmap              : bool
}

/**
 * This structure is used to persist
 * filter specific settings to an INI file.
 */
struct bloom_filter_config {
    initial_capacity      : u64,
    default_probability   : f64,
    scale_size            : i32,
    probability_reduction : f64,
    in_memory             : bool,
    size                  : u64, // Total size
    capacity              : u64, // Total capacity
    bytes                 : u64, // Total byte size
}

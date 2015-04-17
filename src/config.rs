use inifile::IniFile;
use std::str::FromStr;

#[derive(Clone)]
pub struct BloomConfig {
    pub tcp_port              : i32,
    pub udp_port              : i32,
    pub bind_host             : String,
    pub data_dir              : String,
    pub initial_capacity      : u64,
    pub default_probability   : f64,
    pub scale_size            : i32,
    pub probability_reduction : f64,
    pub flush_interval        : i32,
    pub cold_interval         : i32,
    pub in_memory             : bool,
    pub worker_threads        : i32,
    pub use_mmap              : bool
}

const INI_SECTION_BLOOMD               : &'static str = "bloomd";
const INI_OPTION_PORT                  : &'static str = "port";
const INI_OPTION_TCP_PORT              : &'static str = "tcp_port";
const INI_OPTION_UDP_PORT              : &'static str = "udp_port";
const INI_OPTION_SCALE_SIZE            : &'static str = "scale_size";
const INI_OPTION_FLUSH_INTERVAL        : &'static str = "flush_interval";
const INI_OPTION_COLD_INTERVAL         : &'static str = "cold_interval";
const INI_OPTION_IN_MEMORY             : &'static str = "in_memory";
const INI_OPTION_WORKERS               : &'static str = "workers";
const INI_OPTION_USE_MMAP              : &'static str = "use_mmap";
const INI_OPTION_INITIAL_CAPACITY      : &'static str = "initial_capacity";
const INI_OPTION_DEFAULT_PROBABILITY   : &'static str = "default_probability";
const INI_OPTION_PROBABILITY_REDUCTION : &'static str = "probability_reduction";
const INI_OPTION_DATA_DIR              : &'static str = "data_dir";
const INI_OPTION_BIND_ADDRESS          : &'static str = "bind_address";

impl BloomConfig {
    pub fn new (tcp_port              : i32,
                udp_port              : i32,
                bind_host             : &str,
                data_dir              : &str,
                initial_capacity      : u64,
                default_probability   : f64,
                scale_size            : i32,
                probability_reduction : f64,
                flush_interval        : i32,
                cold_interval         : i32,
                in_memory             : bool,
                worker_threads        : i32,
                use_mmap              : bool) -> Self {
        return BloomConfig {
            tcp_port: tcp_port,
            udp_port: udp_port,
            bind_host: String::from_str(bind_host),
            data_dir: String::from_str(data_dir),
            initial_capacity: initial_capacity,
            default_probability: default_probability,
            scale_size: scale_size,
            probability_reduction: probability_reduction,
            flush_interval: flush_interval,
            cold_interval: cold_interval,
            in_memory: in_memory,
            worker_threads: worker_threads,
            use_mmap: use_mmap
        };
    }

    pub fn default() -> Self {
        return BloomConfig::new(
            8673,          // tcp_port
            8674,          // udp_port
            "0.0.0.0",     // bind_host
            "/tmp/bloomd", // data_dir
            100000,        // initial_capacity
            0.0001,        // default_probability
            4,             // scale_size
            0.9,           // probability_reduction
            60,            // flush_interval
            3600,          // cold_interval
            false,         // in_memory
            1,             // worker_threads
            false          // use_mmap
        );
    }

    pub fn from_filename(filename : &str) -> Self {
        let mut config : BloomConfig = BloomConfig::default();

        let ini_opt : Result<IniFile, _> = IniFile::from_filename(filename);

        if ini_opt.is_ok() {
            let ini : IniFile = ini_opt.unwrap();

            if ini.has_section(INI_SECTION_BLOOMD.as_slice()) {
                for option in ini.options(String::from_str(INI_SECTION_BLOOMD)).iter() {
                    match option.as_slice() {
                        INI_OPTION_PORT                  => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_PORT).unwrap() },
                        INI_OPTION_TCP_PORT              => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_TCP_PORT).unwrap() },
                        INI_OPTION_UDP_PORT              => { config.udp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_UDP_PORT).unwrap() },
                        INI_OPTION_SCALE_SIZE            => { config.scale_size            = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_SCALE_SIZE).unwrap() },
                        INI_OPTION_FLUSH_INTERVAL        => { config.flush_interval        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_FLUSH_INTERVAL).unwrap() },
                        INI_OPTION_COLD_INTERVAL         => { config.cold_interval         = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_COLD_INTERVAL).unwrap() },
                        INI_OPTION_WORKERS               => { config.worker_threads        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_WORKERS).unwrap() }
                        INI_OPTION_INITIAL_CAPACITY      => { config.initial_capacity      = ini.get::<u64>(INI_SECTION_BLOOMD, INI_OPTION_INITIAL_CAPACITY).unwrap() },
                        INI_OPTION_USE_MMAP              => { config.use_mmap              = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_USE_MMAP).unwrap() },
                        INI_OPTION_IN_MEMORY             => { config.in_memory             = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_IN_MEMORY).unwrap() },
                        INI_OPTION_DEFAULT_PROBABILITY   => { config.default_probability   = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_DEFAULT_PROBABILITY).unwrap() },
                        INI_OPTION_PROBABILITY_REDUCTION => { config.probability_reduction = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_PROBABILITY_REDUCTION).unwrap() },
                        INI_OPTION_DATA_DIR              => { config.data_dir              = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_DATA_DIR).unwrap() },
                        INI_OPTION_BIND_ADDRESS          => { config.bind_host             = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_BIND_ADDRESS).unwrap() },
                        _ => { panic!("Unknown option: {}", option) }
                    }
                }
            }
        }

        return config;
    }

    pub fn get_bind_address(&self) -> String {
        return format!("{}:{}", self.bind_host, self.tcp_port);
    }
}

unsafe impl Send for BloomConfig { }

const INI_SECTION_CONFIG          : &'static str = "config";
const INI_OPTION_CAPACITY         : &'static str = "capacity";
const INI_OPTION_PROBABILITY      : &'static str = "probability";
const INI_OPTION_K_NUM            : &'static str = "k_num";
const INI_OPTION_BYTES            : &'static str = "bytes";
const INI_OPTION_SIZE             : &'static str = "size";
const INI_OPTION_BITMAP_FILENAMES : &'static str = "bitmap_filenames";
const INI_OPTION_FILTER_SIZES     : &'static str = "filter_sizes";

/**
 * This structure is used to persist
 * filter specific settings to an INI file.
 */
pub struct BloomFilterConfig {
    pub capacity              : u64,         // Total capacity
    pub probability           : f64,         // False positive probability
    pub k_num                 : u32,         // K value
    pub in_memory             : bool,        // Filter is only contained in memory
    pub bytes                 : u64,         // Total byte size
    pub size                  : u64,         // Total size
    pub bitmap_filenames      : Vec<String>, // bitmap filenames
    pub filter_sizes          : Vec<u64>     // filter sizes
}

impl BloomFilterConfig {
    pub fn new(capacity : u64, probability : f64, k_num : u32, in_memory : bool, bytes : u64) -> Self {
        return BloomFilterConfig {
            capacity: capacity,
            probability: probability,
            k_num: k_num,
            in_memory: in_memory,
            bytes: bytes,
            size: 0,
            bitmap_filenames: Vec::new(),
            filter_sizes: Vec::new() };
    }

    pub fn from_ini(ini : &IniFile) -> Result<Self, ()> {
        let capacity : u64;
        match ini.get::<u64>(INI_SECTION_CONFIG, INI_OPTION_CAPACITY) {
            Some(value) => { capacity = value },
            None => { return Err(()) }
        };

        let probability : f64;
        match ini.get::<f64>(INI_SECTION_CONFIG, INI_OPTION_PROBABILITY) {
            Some(value) => { probability = value },
            None => { return Err(()) }
        };

        let k_num : u32;
        match ini.get::<u32>(INI_SECTION_CONFIG, INI_OPTION_K_NUM) {
            Some(value) => { k_num = value },
            None => { return Err(()) }
        };

        let in_memory : bool;
        match ini.get_bool(INI_SECTION_CONFIG, INI_OPTION_IN_MEMORY) {
            Some(value) => { in_memory = value },
            None => { return Err(()) }
        };

        let bytes : u64;
        match ini.get::<u64>(INI_SECTION_CONFIG, INI_OPTION_BYTES) {
            Some(value) => { bytes = value },
            None => { return Err(()) }
        };

        let size : u64;
        match ini.get::<u64>(INI_SECTION_CONFIG, INI_OPTION_SIZE) {
            Some(value) => { size = value },
            None => { return Err(()) }
        };

        let bitmap_filenames : Vec<String>;
        match ini.get_string(INI_SECTION_CONFIG, INI_OPTION_BITMAP_FILENAMES) {
            Some(value) => { bitmap_filenames = value.split_str(",").map(|piece| String::from_str(piece) ).collect() },
            None => { return Err(()) }
        };

        let filter_sizes : Vec<u64>;
        match ini.get_string(INI_SECTION_CONFIG, INI_OPTION_FILTER_SIZES) {
            Some(value) => { filter_sizes = value.split_str(",").map(|piece| FromStr::from_str(piece).unwrap() ).collect::<Vec<u64>>() },
            None => { return Err(()) }
        };

        return Ok(BloomFilterConfig {
            capacity: capacity,
            probability: probability,
            k_num: k_num,
            in_memory: in_memory,
            bytes: bytes,
            size: size,
            bitmap_filenames: bitmap_filenames,
            filter_sizes: filter_sizes
        });
    }
}

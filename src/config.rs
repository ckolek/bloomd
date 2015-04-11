
use inifile::IniFile;

#[derive(Clone)]
pub struct bloom_config {
    tcp_port              : i32,
    udp_port              : i32,
    bind_address          : String,
    data_dir              : String,
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

static DEFAULT_CONFIG : bloom_config = bloom_config {
    tcp_port: 8673,
    udp_port: 8674,
    bind_address: String::from_str("0.0.0.0"),
    data_dir: String::from_str("/tmp/bloomd"),
    initial_capacity: 100000,
    default_probability: 0.0001,
    scale_size: 4,
    probability_reduction: 0.9,
    flush_interval: 60,
    cold_interval: 3600,
    in_memory: false,
    worker_threads: 1,
    use_mmap: false
};

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

impl bloom_config {
    pub fn from_filename(filename : &str) -> Self {
        let mut config : bloom_config = DEFAULT_CONFIG.clone();

        let mut ini : IniFile = IniFile::new();
        ini.read(filename);

        if ini.has_section(INI_SECTION_BLOOMD.as_slice()) {
            for option in ini.options(String::from_str(INI_SECTION_BLOOMD)).iter() {
                match option.as_slice() {
                    INI_OPTION_PORT                  => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_PORT) },
                    INI_OPTION_TCP_PORT              => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_TCP_PORT) },
                    INI_OPTION_UDP_PORT              => { config.udp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_UDP_PORT) },
                    INI_OPTION_SCALE_SIZE            => { config.scale_size            = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_SCALE_SIZE) },
                    INI_OPTION_FLUSH_INTERVAL        => { config.flush_interval        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_FLUSH_INTERVAL) },
                    INI_OPTION_COLD_INTERVAL         => { config.cold_interval         = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_COLD_INTERVAL) },
                    INI_IPTION_WORKERS               => { config.worker_threads        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_WORKERS) },
                    INI_OPTION_INITIAL_CAPACITY      => { config.initial_capacity      = ini.get::<u64>(INI_SECTION_BLOOMD, INI_OPTION_INITIAL_CAPACITY) },
                    INI_OPTION_USE_MMAP              => { config.use_mmap              = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_USE_MMAP) },
                    INI_OPTION_IN_MEMORY             => { config.in_memory             = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_IN_MEMORY) },
                    INI_OPTION_DEFAULT_PROBABILITY   => { config.default_probability   = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_DEFAULT_PROBABILITY) },
                    INI_OPTION_PROBABILITY_REDUCTION => { config.probability_reduction = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_PROBABILITY_REDUCTION) },
                    INI_OPTION_DATA_DIR              => { config.data_dir              = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_DATA_DIR) },
                    INI_OPTION_BIND_ADDRESS          => { config.bind_address          = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_BIND_ADDRESS) }
                }
            }
        }

        return config;
    }
}

/**
 * This structure is used to persist
 * filter specific settings to an INI file.
 */
pub struct bloom_filter_config {
    initial_capacity      : u64,
    default_probability   : f64,
    scale_size            : i32,
    probability_reduction : f64,
    in_memory             : bool,
    size                  : u64, // Total size
    capacity              : u64, // Total capacity
    bytes                 : u64, // Total byte size
}

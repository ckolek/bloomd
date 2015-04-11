
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
    bind_address: "0.0.0.0",
    data_dir: "/tmp/bloomd",
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

static INI_SECTION_BLOOMD               : &'static str = "bloomd";
static INI_OPTION_PORT                  : &'static str = "port";
static INI_OPTION_TCP_PORT              : &'static str = "tcp_port";
static INI_OPTION_UDP_PORT              : &'static str = "udp_port";
static INI_OPTION_SCALE_SIZE            : &'static str = "scale_size";
static INI_OPTION_FLUSH_INTERVAL        : &'static str = "flush_interval";
static INI_OPTION_COLD_INTERVAL         : &'static str = "cold_interval";
static INI_OPTION_IN_MEMORY             : &'static str = "in_memory";
static INI_OPTION_WORKERS               : &'static str = "workers";
static INI_OPTION_USE_MMAP              : &'static str = "use_mmap";
static INI_OPTION_INITIAL_CAPACITY      : &'static str = "initial_capacity";
static INI_OPTION_DEFAULT_PROBABILITY   : &'static str = "default_probability";
static INI_OPTION_PROBABILITY_REDUCTION : &'static str = "probability_reduction";
static INI_OPTION_DATA_DIR              : &'static str = "data_dir";
static INI_OPTION_BIND_ADDRESS          : &'static str = "bind_address";

impl bloom_config {
    pub fn from_filename(filename : &str) -> Self {
        let mut config : bloom_config = DEFAULT_CONFIG.clone();

        let mut ini : IniFile = IniFile::new();
        ini.read(filename);

        if ini.has_section(INI_SECTION_BLOOMD) {
            for option in ini.options(INI_SECTION_BLOOMD).iter() {
                match option {
                    INI_OPTION_PORT                  => { config.tcp_port              = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_PORT) },
                    INI_OPTION_TCP_PORT              => { config.tcp_port              = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_TCP_PORT) },
                    INI_OPTION_UDP_PORT              => { config.udp_port              = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_UDP_PORT) },
                    INI_OPTION_SCALE_SIZE            => { config.scale_size            = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_SCALE_SIZE) },
                    INI_OPTION_FLUSH_INTERVAL        => { config.flush_interval        = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_FLUSH_INTERVAL) },
                    INI_OPTION_COLD_INTERVAL         => { config.cold_interval         = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_COLD_INTERVAL) },
                    INI_IPTION_WORKERS               => { config.worker_threads        = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_WORKERS) },
                    INI_OPTION_INITIAL_CAPACITY      => { config.initial_capacity      = ini.get_int( INI_SECTION_BLOOMD, INI_OPTION_INITIAL_CAPACITY) },
                    INI_OPTION_USE_MMAP              => { config.use_mmap              = ini.get_bool(INI_SECTION_BLOOMD, INI_OPTION_USE_MMAP) },
                    INI_OPTION_IN_MEMORY             => { config.in_memory             = ini.get_bool(INI_SECTION_BLOOMD, INI_OPTION_IN_MEMORY) },
                    INI_OPTION_DEFAULT_PROBABILITY   => { config.default_probabiity    = ini.get_f64( INI_SECTION_BLOOMD, INI_OPTION_DEFAULT_PROBABILITY) },
                    INI_OPTION_PROBABILITY_REDUCTION => { config.probability_reduction = ini.get_f64( INI_SECTION_BLOOMD, INI_OPTION_PROBABILITY_REDUCTION) },
                    INI_OPTION_DATA_DIR              => { config.data_dir              = ini.get(     INI_SECTION_BLOOMD, INI_OPTION_DATA_DIR) },
                    INI_OPTION_BIND_ADDRESS          => { config.bind_address          = ini.get(     INI_SECTION_BLOOMD, INI_OPTION_BIND_ADDRESS) }
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

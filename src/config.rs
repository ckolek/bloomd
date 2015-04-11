use inifile::IniFile;

#[derive(Clone)]
pub struct BloomConfig {
    pub tcp_port              : i32,
    pub udp_port              : i32,
    pub bind_address          : String,
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
                bind_address          : &str,
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
            bind_address: String::from_str(bind_address),
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
            "0.0.0.0",     // bind_address
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
                        INI_OPTION_PORT                  => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_PORT) },
                        INI_OPTION_TCP_PORT              => { config.tcp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_TCP_PORT) },
                        INI_OPTION_UDP_PORT              => { config.udp_port              = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_UDP_PORT) },
                        INI_OPTION_SCALE_SIZE            => { config.scale_size            = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_SCALE_SIZE) },
                        INI_OPTION_FLUSH_INTERVAL        => { config.flush_interval        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_FLUSH_INTERVAL) },
                        INI_OPTION_COLD_INTERVAL         => { config.cold_interval         = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_COLD_INTERVAL) },
                        INI_OPTION_WORKERS               => { config.worker_threads        = ini.get::<i32>(INI_SECTION_BLOOMD, INI_OPTION_WORKERS) }
                        INI_OPTION_INITIAL_CAPACITY      => { config.initial_capacity      = ini.get::<u64>(INI_SECTION_BLOOMD, INI_OPTION_INITIAL_CAPACITY) },
                        INI_OPTION_USE_MMAP              => { config.use_mmap              = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_USE_MMAP) },
                        INI_OPTION_IN_MEMORY             => { config.in_memory             = ini.get_bool  (INI_SECTION_BLOOMD, INI_OPTION_IN_MEMORY) },
                        INI_OPTION_DEFAULT_PROBABILITY   => { config.default_probability   = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_DEFAULT_PROBABILITY) },
                        INI_OPTION_PROBABILITY_REDUCTION => { config.probability_reduction = ini.get::<f64>(INI_SECTION_BLOOMD, INI_OPTION_PROBABILITY_REDUCTION) },
                        INI_OPTION_DATA_DIR              => { config.data_dir              = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_DATA_DIR) },
                        INI_OPTION_BIND_ADDRESS          => { config.bind_address          = ini.get_string(INI_SECTION_BLOOMD, INI_OPTION_BIND_ADDRESS) },
                        _ => { panic!("Unknown option: {}", option) }
                    }
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
pub struct BloomFilterConfig {
    pub initial_capacity      : u64,
    pub default_probability   : f64,
    pub scale_size            : i32,
    pub probability_reduction : f64,
    pub in_memory             : bool,
    pub size                  : u64, // Total size
    pub capacity              : u64, // Total capacity
    pub bytes                 : u64, // Total byte size
}

// TODO: Work out what other options here
impl BloomFilterConfig {
    pub fn new(initial_capacity : u64, default_probability : f64) -> Self {
        return BloomFilterConfig {
            initial_capacity      : initial_capacity,
            default_probability   : default_probability,
            scale_size            : 0,
            probability_reduction : default_probability,
            in_memory             : true,
            size                  : 0,
            capacity              : initial_capacity,
            bytes                 : 0
        }
    }
}

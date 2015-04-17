use inifile::IniFile;
use config::BloomFilterConfig;
use bloom::{bloom_filter_params, bloom_bloomfilter, load_bloom_filter};
use lbf::bloom_lbf;
use std::ops::{Deref, DerefMut};
use std::io::fs::PathExtensions;

const INI_SECTION_COUNTERS : &'static str = "counters";
const INI_OPTION_CHECK_HITS : &'static str = "check_hits";
const INI_OPTION_CHECK_MISSES : &'static str = "check_misses";
const INI_OPTION_SET_HITS : &'static str = "set_hits";
const INI_OPTION_SET_MISSES : &'static str = "set_misses";
const INI_OPTION_PAGE_INS : &'static str = "page_ins";
const INI_OPTION_PAGE_OUTS : &'static str = "page_outs";

pub struct BloomFilterCounters {
    pub check_hits   : u64,
    pub check_misses : u64,
    pub set_hits     : u64,
    pub set_misses   : u64,
    pub page_ins     : u64,
    pub page_outs    : u64
}

impl BloomFilterCounters {
    pub fn new() -> Self {
        return BloomFilterCounters { check_hits: 0, check_misses: 0, set_hits: 0, set_misses: 0, page_ins: 0, page_outs: 0 };
    }

    pub fn from_ini(ini : &IniFile) -> Result<Self, ()> {
        let check_hits : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_CHECK_HITS) {
            Some(value) => { check_hits = value },
            None => { return Err(()) }
        };

        let check_misses : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_CHECK_MISSES) {
            Some(value) => { check_misses = value },
            None => { return Err(()) }
        };

        let set_hits : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_SET_HITS) {
            Some(value) => { set_hits = value },
            None => { return Err(()) }
        };

        let set_misses : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_SET_MISSES) {
            Some(value) => { set_misses = value },
            None => { return Err(()) }
        };

        let page_ins : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_PAGE_INS) {
            Some(value) => { page_ins = value },
            None => { return Err(()) }
        };

        let page_outs : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_PAGE_OUTS) {
            Some(value) => { page_outs = value },
            None => { return Err(()) }
        };

        return Ok(BloomFilterCounters {
            check_hits: check_hits,
            check_misses: check_misses,
            set_hits: set_hits,
            set_misses: set_misses,
            page_ins: page_ins,
            page_outs: page_outs
        });
    } 

    pub fn checks(&self) -> u64 {
        return self.check_hits + self.check_misses;
    }

    pub fn sets(&self) -> u64 {
        return self.set_hits + self.set_misses;
    }
}

pub struct BloomFilter {
    pub config      : BloomFilterConfig,   // Filter-specific config
    pub lbf         : bloom_lbf,           // Layered bloom filter
    pub counters    : BloomFilterCounters, // Counters
    pub directory   : Path,                // File directory path,
    pub config_file : Path                 // INI file path 
}

impl BloomFilter {
    pub fn new(config : BloomFilterConfig, lbf : bloom_lbf, directory : Path) -> BloomFilter {
        let mut config_file : Path = directory.clone();
        config_file.push(lbf.name.as_slice());

        return BloomFilter {
            config      : config,
            lbf         : lbf,
            counters    : BloomFilterCounters::new(),
            directory   : directory,
            config_file : config_file
        };
    }

    pub fn from_directory(directory : Path, filter_name : &String) -> Result<Self, ()> {
        if directory.exists() {
            let mut config_file : Path = directory.clone();
            config_file.push(filter_name.as_slice());
            config_file.set_extension("ini");

            return match IniFile::from_filename(config_file.as_str().unwrap()) {
                Ok(ini) => {
                    let config : BloomFilterConfig;
                    match BloomFilterConfig::from_ini(&ini) {
                        Ok(_config) => { config = _config },
                        Err(_) => { return Err(()) }
                    };

                    let params : bloom_filter_params = bloom_filter_params::new(config.bytes, config.k_num, config.capacity, config.probability);

                    let mut filters : Vec<bloom_bloomfilter> = Vec::new();
                    for bitmap_filename in config.bitmap_filenames.iter() {
                        let index : usize = filters.len();

                        filters.push(load_bloom_filter(&params, config.filter_sizes[index], bitmap_filename.as_slice()));
                    }

                    let lbf : bloom_lbf = bloom_lbf::new(params, filter_name.clone(), filters);

                    let counters : BloomFilterCounters;
                    match BloomFilterCounters::from_ini(&ini) {
                        Ok(_counters) => { counters = _counters },
                        Err(_) => { return return Err(()) }
                    };

                    return Ok(BloomFilter {
                        config: config,
                        lbf: lbf,
                        counters: counters,
                        directory: directory,
                        config_file: config_file
                    });
                },
                Err(_) => { Err(()) }
            };
        }

        return Err(());
    }

    pub fn flush() -> Result<(), ()> {
        let mut ini : IniFile = IniFile::new();

        return Ok(());
    }
}

impl Deref for BloomFilter {
    type Target = bloom_lbf;

    fn deref<'a>(&'a self) -> &'a bloom_lbf {
        return &self.lbf;
    }
}

impl DerefMut for BloomFilter {
    fn deref_mut<'a>(&'a mut self) -> &'a mut bloom_lbf {
        return &mut self.lbf;
    }
}

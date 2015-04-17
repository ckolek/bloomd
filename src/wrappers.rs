use inifile::IniFile;
use config::BloomFilterConfig;
use filter::IBloomFilter;
use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter, load_bloom_filter};
use lbf::bloom_lbf;
use std::ops::{Deref, DerefMut};
use std::io::fs;
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

    pub fn add_to_ini(&self, ini : &mut IniFile) {
        ini.add_section(INI_SECTION_COUNTERS);
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_CHECK_HITS, self.check_hits.to_string());
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_CHECK_MISSES, self.check_misses.to_string());
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_SET_HITS, self.set_hits.to_string());
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_SET_MISSES, self.set_misses.to_string());
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_PAGE_INS, self.page_ins.to_string());
        ini.set(INI_SECTION_COUNTERS, INI_OPTION_PAGE_OUTS, self.page_outs.to_string());
    }
}

pub struct BloomFilter {
    pub config      : BloomFilterConfig,   // Filter-specific config
    lbf             : Option<bloom_lbf>,   // Layered bloom filter
    pub counters    : BloomFilterCounters, // Counters
    pub directory   : Path,                // File directory path,
    pub config_file : Path                 // INI file path 
}

impl BloomFilter {
    pub fn new(config : BloomFilterConfig, lbf : bloom_lbf, directory : Path) -> BloomFilter {
        let mut config_file : Path = directory.clone();
        config_file.push(lbf.name.as_slice());
        config_file.set_extension("ini");

        return BloomFilter {
            config      : config,
            lbf         : Some(lbf),
            counters    : BloomFilterCounters::new(),
            directory   : directory,
            config_file : config_file
        };
    }

    pub fn from_directory(directory : &Path, filter_name : &String) -> Result<Self, ()> {
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

                    let counters : BloomFilterCounters;
                    match BloomFilterCounters::from_ini(&ini) {
                        Ok(_counters) => { counters = _counters },
                        Err(_) => { return return Err(()) }
                    };

                    let mut bloom_filter : BloomFilter = BloomFilter {
                        config: config,
                        lbf: None,
                        counters: counters,
                        directory: directory.clone(),
                        config_file: config_file
                    };
                    bloom_filter.load_filter();

                    return Ok(bloom_filter);
                },
                Err(_) => { Err(()) }
            };
        }

        return Err(());
    }

    pub fn add_filter(&mut self, value : u32) {
        let mut path : Path = self.directory.clone();
        path.push(value.to_string());
        path.set_extension("bmp");

        let bitmap_filename : String = String::from_str(path.as_str().unwrap());

        let bloom_filter : bloom_bloomfilter = create_bloom_filter(&(**self).params, bitmap_filename.as_slice());

        (**self).add_filter(bloom_filter);
        self.config.bitmap_filenames.push(bitmap_filename);
        self.config.filter_sizes.push(0);
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        let mut ini : IniFile = IniFile::new();
        self.config.add_to_ini(&mut ini);
        self.counters.add_to_ini(&mut ini);

        if ini.write_to_path(&self.config_file).is_err() {
            println!("Could not write to config file: {}", self.config_file.as_str().unwrap());
        } 

        return (**self).flush();
    }

    fn load_filter(&mut self) {
        let params : bloom_filter_params = bloom_filter_params::new(self.config.bytes, self.config.k_num, self.config.capacity, self.config.probability);

        let mut filters : Vec<bloom_bloomfilter> = Vec::new();
        for bitmap_filename in self.config.bitmap_filenames.iter() {
            let index : usize = filters.len();

            filters.push(load_bloom_filter(&params, self.config.filter_sizes[index], bitmap_filename.as_slice()));
        }

        let lbf : bloom_lbf = bloom_lbf::new(params, self.config.filter_name.clone(), filters);

        self.lbf = Some(lbf);
    }

    pub fn unload_filter(&mut self) {
        self.lbf = None;
    }

    pub fn delete(&mut self) {
        fs::rmdir_recursive(&self.directory).unwrap();
    }
}

impl Deref for BloomFilter {
    type Target = bloom_lbf;

    fn deref<'a>(&'a self) -> &'a bloom_lbf {
        return self.lbf.as_ref().unwrap();
    }
}

impl DerefMut for BloomFilter {
    fn deref_mut<'a>(&'a mut self) -> &'a mut bloom_lbf {
        if self.lbf.is_none() {
            self.load_filter();
        }

        return self.lbf.as_mut().unwrap();
    }
}

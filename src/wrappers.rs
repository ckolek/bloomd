use inifile::IniFile;
use config::BloomFilterConfig;
use filter::IBloomFilter;
use bloom::{bloom_filter_params, bloom_bloomfilter, create_bloom_filter, load_bloom_filter};
use lbf::bloom_lbf;
use std::ops::{Deref, DerefMut};
use std::io;
use std::io::{fs, IoResult};
use std::io::fs::PathExtensions;

// constants -------------------------------------------------------------------
const INI_SECTION_COUNTERS : &'static str = "counters";
const INI_OPTION_CHECK_HITS : &'static str = "check_hits";
const INI_OPTION_CHECK_MISSES : &'static str = "check_misses";
const INI_OPTION_SET_HITS : &'static str = "set_hits";
const INI_OPTION_SET_MISSES : &'static str = "set_misses";
const INI_OPTION_PAGE_INS : &'static str = "page_ins";
const INI_OPTION_PAGE_OUTS : &'static str = "page_outs";
// -----------------------------------------------------------------------------

// Keeps track of statistics for filters; used by the info command
pub struct BloomFilterCounters {
    pub check_hits   : u64,
    pub check_misses : u64,
    pub set_hits     : u64,
    pub set_misses   : u64,
    pub page_ins     : u64,
    pub page_outs    : u64
}

impl BloomFilterCounters {
    // Returns a new instance, with all counters set to zero
    pub fn new() -> Self {
        return BloomFilterCounters { check_hits: 0, check_misses: 0, set_hits: 0, set_misses: 0, page_ins: 0, page_outs: 0 };
    }

    // Loads an instance from an ini file, returning an error if the ini file is missing information
    pub fn from_ini(ini : &IniFile) -> Result<Self, String> {
        let check_hits : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_CHECK_HITS) {
            Some(value) => { check_hits = value },
            None => { return Err(String::from_str("missing counters:check_hits")) }
        };

        let check_misses : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_CHECK_MISSES) {
            Some(value) => { check_misses = value },
            None => { return Err(String::from_str("missing counters:check_misses")) }
        };

        let set_hits : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_SET_HITS) {
            Some(value) => { set_hits = value },
            None => { return Err(String::from_str("missing counters:set_hits")) }
        };

        let set_misses : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_SET_MISSES) {
            Some(value) => { set_misses = value },
            None => { return Err(String::from_str("missing counters:set_misses")) }
        };

        let page_ins : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_PAGE_INS) {
            Some(value) => { page_ins = value },
            None => { return Err(String::from_str("missing counters:page_ins")) }
        };

        let page_outs : u64;
        match ini.get::<u64>(INI_SECTION_COUNTERS, INI_OPTION_PAGE_OUTS) {
            Some(value) => { page_outs = value },
            None => { return Err(String::from_str("missing counters:page_outs")) }
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

    // Returns the aggregate number of checks
    pub fn checks(&self) -> u64 {
        return self.check_hits + self.check_misses;
    }

    // Returns the aggregate number of sets
    pub fn sets(&self) -> u64 {
        return self.set_hits + self.set_misses;
    }

    // Adds the counters in this instance to the given ini file
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

// The structure wrapping around bloom filters, used by bloomd.
// Keeps the configuration, the filter, and the counters
pub struct BloomFilter {
    pub config      : BloomFilterConfig,   // Filter-specific config
    lbf             : Option<bloom_lbf>,   // Layered bloom filter
    pub counters    : BloomFilterCounters, // Counters
    pub directory   : Path,                // File directory path,
    pub config_file : Path,                // INI file path
    pub cold_index  : u64                  // Used to determine how recently filter was used
}

impl BloomFilter {
    // Returns a new instance of a BloomFilter
    pub fn new(config : BloomFilterConfig, lbf : bloom_lbf, directory : Path) -> BloomFilter {
        let mut config_file : Path = directory.clone();
        config_file.push(lbf.name.as_slice());
        config_file.set_extension("ini");

        return BloomFilter {
            config      : config,
            lbf         : Some(lbf),
            counters    : BloomFilterCounters::new(),
            directory   : directory,
            config_file : config_file,
            cold_index  : 0
        };
    }

    // Reads in a Bloom Filter from a given directory; returns an error if
    // the ini file is missing or lacks information
    pub fn from_directory(directory : &Path, filter_name : &String, load_filter : bool) -> Result<Self, String> {
        if directory.exists() {
            let mut config_file : Path = directory.clone();
            config_file.push(filter_name.as_slice());
            config_file.set_extension("ini");

            return match IniFile::from_filename(config_file.as_str().unwrap()) {
                Ok(ini) => {
                    let config : BloomFilterConfig;
                    match BloomFilterConfig::from_ini(&ini) {
                        Ok(_config) => { config = _config },
                        Err(e) => { return Err(e) }
                    };

                    let counters : BloomFilterCounters;
                    match BloomFilterCounters::from_ini(&ini) {
                        Ok(_counters) => { counters = _counters },
                        Err(e) => { return return Err(e) }
                    };

                    let mut bloom_filter : BloomFilter = BloomFilter {
                        config: config,
                        lbf: None,
                        counters: counters,
                        directory: directory.clone(),
                        config_file: config_file,
                        cold_index: 0
                    };

                    if load_filter {
                        bloom_filter.load_filter();
                    }

                    return Ok(bloom_filter);
                },
                Err(e) => { Err(e.to_string()) }
            };
        }

        return Err(format!("directory {} does not exist", directory.display()));
    }

    // Handles the creation of a bloom filter on the disk, including the corresponding bitmap
    pub fn add_filter(&mut self, value : u32) -> Result<(), String> {
        let mut path : Path = self.directory.clone();
        path.push(value.to_string());
        path.set_extension("bmp");

        let bitmap_filename : String = String::from_str(path.as_str().unwrap());

        let bloom_filter : bloom_bloomfilter;
        match create_bloom_filter(&(**self).params, bitmap_filename.as_slice(), self.config.in_memory) {
            Ok(_bloom_filter) => { bloom_filter = _bloom_filter },
            Err(e) => { return Err(e) }
        }

        (**self).add_filter(bloom_filter);

        if !self.config.in_memory {
            self.config.bitmap_filenames.push(bitmap_filename);
        }
        self.config.filter_sizes.push(0);

        return Ok(());
    }

    // Flushes the bloom filter back to the disk
    pub fn flush(&mut self) -> Result<(), String> {
        if !self.config.in_memory {
            let mut ini : IniFile = IniFile::new();
            self.config.add_to_ini(&mut ini);
            self.counters.add_to_ini(&mut ini);

            if ini.write_to_path(&self.config_file).is_err() {
                println!("Could not write to config file: {}", self.config_file.as_str().unwrap());
            } 
        }

        return (**self).flush();
    }

    // Loads a bloom filter from the disk back into the BloomFilter instance
    fn load_filter(&mut self) {
        let params : bloom_filter_params = bloom_filter_params::new(self.config.bytes, self.config.k_num, self.config.capacity, self.config.probability);

        let mut filters : Vec<bloom_bloomfilter> = Vec::new();
        for bitmap_filename in self.config.bitmap_filenames.iter() {
            let index : usize = filters.len();

            match load_bloom_filter(&params, self.config.filter_sizes[index], bitmap_filename.as_slice(), self.config.in_memory) {
                Ok(filter) => { filters.push(filter) },
                Err(e) => { println!("Could not load filter {} ({}): {}", index, bitmap_filename, e) }
            }
        }

        let lbf : bloom_lbf = bloom_lbf::new(params, self.config.filter_name.clone(), filters);

        self.lbf = Some(lbf);
    }

    // Removes the bloom filter from memory
    pub fn unload_filter(&mut self) {
        self.lbf = None;
    }

    // Resets the cold index for this filter
    pub fn touch(&mut self) {
        self.cold_index = 0;
        (**self).touch();
    }

    // Initializes the bloom filter on disk
    pub fn init(&self) -> IoResult<()>{
        if !self.config.in_memory {
            fs::mkdir(&self.directory, io::USER_RWX).unwrap();
        }

        return Ok(());
    }

    // Deletes the bloom filter from the disk
    pub fn delete(&mut self) -> IoResult<()> {
        if self.directory.exists() {
            return fs::rmdir_recursive(&self.directory);
        }

        return Ok(());
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

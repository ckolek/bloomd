//! INI file management, partial implementation of Python API.
//!
//! Tested with rust-0.12-pre
//!
//! @author Eliovir <http://github.com/~eliovir>
//!
//! @license MIT license <http://www.opensource.org/licenses/mit-license.php>
//!
//! @see http://docs.python.org/2/library/configparser.html
//!
//! @since 2013-12-18
//!
//! @todo eddyb: you may want that to be Option<&'a str> so you can return None when the option isn't present. Option<T> can be either Some(T) or None. Option<~T> and Option<&T> are nullable pointers semantically (and optimized as such)

extern crate test;

use std::collections::HashMap;
use std::str::FromStr;
use std::io::{BufferedReader, IoError, IoResult};
use std::io::fs::File;
use std::fmt;

pub struct IniFile {
	/// Comments on sections and options
	comments: HashMap<String, HashMap<String, String>>,
	/// Option names, used to keep order (as HashMap doesn't).
	options: Vec<Vec<String>>,
	/// INI structure: sections contain options (name=>value)
	opts: HashMap<String, HashMap<String, String>>,
	/// Section names, used to keep order (as HashMap doesn't).
	sections: Vec<String>
}

/**
 * IniFile implements a basic configuration which provides a structure similar to what's found in Microsoft Windows INI files.
 * You can use this to write programs which can be customized by end users easily.
 */
impl IniFile {
	/**
	 * Add a section named section to the instance.
	 * If a section by the given name already exists, panic!()
	 */
	pub fn add_section(&mut self, section: &str) {
		if !self.has_section(section) {
			self.comments.insert(section.to_string(), HashMap::new());
			self.opts.insert(section.to_string(), HashMap::new());
			self.sections.push(section.to_string());
			self.options.push(Vec::new());
		} else {
			panic!("The section {:?} already exists!", section);
		}
	}
	/**
	 * Get an option value for the named section.
	 */
	//pub fn get<'a>(&self, section: &'a str, option: &'a str) -> String {
	pub fn get_string(&self, section: &str, option: &str) -> Option<String> {
		if self.has_option(section, option) {
            return Some(self.opts[section.to_string()][option.to_string()].to_string());
		}

        return None;
	}
	/**
	 * A convenience method which coerces the option in the specified section to a boolean.
	 * Note that the accepted values for the option are '1', 'yes', 'true', and 'on', which cause this method to return True, and '0', 'no', 'false', and 'off', which cause it to return False.
	 * @todo These string values are checked in a case-insensitive manner.
	 */
	pub fn get_bool(&self, section: &str, option: &str) -> Option<bool> {
		return match self.get_string(section, option) {
            Some(value) => {
                match value.as_slice() {
                    "1" | "yes" | "true" | "T" | "on" => Some(true),
                    "0" | "no" | "false" | "F" | "off" => Some(false),
                    _ => None
                }
            },
            None => None
        };
	}
	/**
	 * A convenience method which coerces the option in the specified section to an integer.
	 */
	pub fn get<S : FromStr>(&self, section: &str, option: &str) -> Option<S> {
		return match self.get_string(section, option) {
            Some(value) => { FromStr::from_str(value.as_slice()) },
            None => None
        };
	}
	/**
	 * Indicates whether the given section exists and contains the given option.
	 */
	pub fn has_option(&self, section: &str, option: &str) -> bool {
		self.has_section(section) &&
			self.opts[section.to_string()].contains_key(&option.to_string())
	}
	/**
	 * Indicates whether the named section is present in the configuration.
	 */
	pub fn has_section(&self, section: &str) -> bool {
		self.opts.contains_key(&section.to_string())
	}
	pub fn new() -> IniFile {
		IniFile { comments: HashMap::new(), options: Vec::new(), opts: HashMap::new(), sections: Vec::new() }
	}

    pub fn from_filename(filename : &str) -> IoResult<Self> {
        let mut ini_file : IniFile = IniFile::new();
        
        return match ini_file.read(filename) {
            Ok(_)  => Ok(ini_file),
            Err(e) => Err(e)
        };
    }
	/**
	 * Return a list of options available in the specified section.
	 */
	pub fn options(&self, section: String) -> Vec<String> {
		match self.sections.as_slice().position_elem(&section) {
			Some(section_index) => self.options[section_index].to_vec(),
			None => {
				vec!()
			},
		}
	}
	/**
	 * Read and parse configuration data from filepath.
	 */
	pub fn read(&mut self, filepath: &str) -> Result<(), IoError> {
		let file = File::open(&Path::new(filepath));
		match file {
			Err(e) => {return Err(e)},
			_ => { }
		}
		let mut reader = BufferedReader::new(file);
		let mut lines: Vec<String> = Vec::new();
		for line in reader.lines() {
			match line {
				Ok(nread) => lines.push(nread),
				Err(e) => println!("error reading: {}", e)
			}
		}
		self.read_string(lines);

        return Ok(());
	}
	/**
	 * Parse configuration data from a vector of strings (file lines).
	 */
	pub fn read_string(&mut self, lines: Vec<String>) {
		let mut section: String = "Default".to_string();
		let mut comment_lines = String::new();
		for line in lines.iter() {
			let mut line_len = line.len();
			let line_slice = line.as_slice();
			if line_len > 0 && line_slice.slice_chars(line_len - 1, line_len) == "\n" {
				line_len = line_len - 1;
			}
			if line_len == 0 {
				comment_lines.push_str(line_slice);
				continue;
			}
			if line_slice.slice_chars(0, 1) == "#" ||
			line_slice.slice_chars(0, 1) == ";" {
				comment_lines.push_str(line_slice);
				continue;
			}
			if line_slice.slice_chars(0, 1) == "[" {
				section = line_slice.slice_chars(1, line_len - 1).to_string();
				if !self.opts.contains_key(&section) {
					self.add_section(section.as_slice());
					self.comments.get_mut(&section).unwrap().insert("__section_comment__".to_string(), comment_lines.clone());
					comment_lines = String::new();
				}
				continue;
			}
			let index = line_slice.find_str("=").unwrap();
			let optkey = line_slice.slice_chars(0, index).to_string();
			let optval = line_slice.slice_chars(index + 1, line_len).to_string();
			self.comments.get_mut(&section).unwrap().insert(optkey.clone(), comment_lines.clone());
			comment_lines = String::new();
			self.opts.get_mut(&section).unwrap().insert(optkey.clone(), optval);
			let section_index = self.sections.as_slice().position_elem(&section).unwrap();
			self.options.get_mut(section_index).unwrap().push(optkey.clone());
		}
	}
	/**
	 * Remove the specified option from the specified section. If the section does not exist, fails.
	 * If the option existed to be removed, return True; otherwise return False.
	 */
	 pub fn remove_option(&mut self, section: String, option: String) -> bool {
		if !self.has_section(section.as_slice()) {
			panic!("Section [{:?}] does not exist!");
		}
	/*
		if !self.has_option(section.to_string(), option.to_string()) {
			false
		}
	*/
		let section_index = self.sections.as_slice().position_elem(&section).unwrap();
		self.options.get_mut(section_index).unwrap().remove(section_index);
		self.comments.get_mut(&section).unwrap().remove(&option);
		self.opts.get_mut(&section).unwrap().remove(&option);
	 	true
	 }
	/**
	 * Remove the specified section from the configuration.
	 * If the section in fact existed, return True; otherwise return False.
	 */
	pub fn remove_section(&mut self, section: String) -> bool {
	/*
		if (!self.has_section(section.clone())) {
			false
		}
	*/
		self.opts.remove(&section);
		self.comments.remove(&section);
		match self.sections.as_slice().position_elem(&section) {
			Some(index) => {
				self.sections.remove(index);
				self.options.remove(index);
				true
			},
			None => false
		}
	}
	/**
	 * Return a list of the available sections.
	 */
	pub fn sections(&self) -> Vec<String> {
		/*
		let mut sections: ~[~str] = ~[];
		self.opts.iter().advance(|(k, _)| {sections.push(k.to_string()); true});
		sections
		*/
		self.sections.clone()
	}
	/**
	 * If the given section exists, set the given option to the specified value; otherwise panic!().
	 */
	pub fn set(&mut self, section: &str, option: &str, value: String) {
		if !self.has_section(section) {
			panic!("Section [{:?}] does not exist!");
		}

		if !self.has_option(section, option) {
            let section = String::from_str(section);

			self.opts.get_mut(&section).unwrap().insert(String::from_str(option), value);
			let section_index = self.sections.as_slice().position_elem(&section).unwrap();
			self.options.get_mut(section_index).unwrap().push(String::from_str(option));
		} else {
            let section = String::from_str(section);

			self.opts.get_mut(&section).unwrap().insert(String::from_str(option), value);
		}
	}
	/**
	 * Write a representation of the configuration to the specified file path.
	 * This representation can be parsed by a future read() call.
	 */
	pub fn write_to_filename(&self, filename: &str) -> IoResult<()> {
        return self.write_to_path(&Path::new(filename));
	}

    pub fn write_to_path(&self, filepath: &Path) -> IoResult<()> {
		let mut file = File::create(filepath);
		return file.write(format!("{}", self).as_bytes());
    }
}

/**
 * Operator overloading
 * @see http://maniagnosis.crsr.net/2013/04/operator-overloading-in-rust.html
 */
impl fmt::String for IniFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut lines = String::new();
		let sections = self.sections().clone();
		for section in sections.iter() {
			if self.comments.contains_key(section) && self.comments[*section].contains_key(& "__section_comment__".to_string()) {
				lines.push_str(self.comments[*section]["__section_comment__".to_string()].as_slice());
			}
			let line = format!("[{}]\n", section);
			lines.push_str(line.as_slice());
			let options = self.options(section.clone()).clone();
			for key in options.iter() {
				if self.comments.contains_key(section) && self.comments[*section].contains_key(key) {
					lines.push_str(self.comments[*section][*key].as_slice());
				}
				lines.push_str(format!("{}={}\n", key.to_string(), self.get_string(section.as_slice(), key.as_slice()).unwrap()).as_slice());
			}
		}
		write!(f, "{}", lines)
	}
}

/*
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
	use std::io::BufferedReader;
	use std::io::fs;
	use std::io::fs::File;
    use std::io::fs::PathExtensions;
	#[test]
	fn default_filepath_is_empty() {
		let ini = super::IniFile::new();
		let expected = ".".to_string();
		let found = ini.filepath();
		assert!(expected == found, format!("Default file path must be \"\", not \"{}\".", found));
	}
	#[test]
	fn filepath() {
		let mut ini = super::IniFile::new();
		let filepath = "data/config.ini";
		ini.read(filepath);
		let expected = "data/config.ini".to_string();
		let found=ini.filepath();
		assert!(expected == found, format!("Default file path must be \"{}\", not \"{}\".", expected, found));
	}
	#[test]
	fn sections_length() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let expected = 5;
		let sections = ini.sections();
		let found = sections.len();
		assert!(expected == found, format!("{} sections are expected, not {}.", expected, found));
	}
	#[test]
	fn sections_names() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let expected = vec!["section1".to_string(),  "section2".to_string(),  "Booleans".to_string(),  "Integers".to_string(),  "Floats".to_string()];
		let found = ini.sections();
		assert!(expected == found, format!("Sections must be \"{:?}\", not {:?}.", expected, found));
	}
	#[test]
	fn has_option_true() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let found = ini.has_option("section1", "value11");
		assert!(found, "Option \"value11\" in section [section1] must be found!");
	}
	#[test]
	fn has_option_false() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let found = ini.has_option("section1", "unknown key");
		assert!(!found, "Option \"unknown key\" in section [section1] must not be found!");
	}
	#[test]
	fn has_section_true() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let found = ini.has_section("section1");
		assert!(found, "Section section1 must be found!");
	}
	#[test]
	fn has_section_false() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let found = ini.has_section("unknown section");
		assert!(!found, "Section \"unknown section\" must not be found!");
	}
	#[test]
	fn get() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let expected = "string 11".to_string();
		let found = ini.get("section1", "value11");
		assert!(expected == found, format!("[section1] value11 must be \"{}\", not \"{}\".", expected, found));
	}
	#[test]
	fn get_bool_true() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let options = ["true1", "true2", "true3"];
		for key in options.iter() {
			let found = ini.get_bool("Booleans", key.as_slice());
			assert!(found, format!("[Booleans] {:?} must be true.", key));
		}
	}
	#[test]
	fn get_bool_false() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let options = ["false1", "false2", "false3"];
		for key in options.iter() {
			let found = ini.get_bool("Booleans", key.as_slice());
			assert!(!found, format!("[Booleans] {:?} must be false.", key));
		}
	}
	#[test]
	#[should_fail]
	fn get_bool_fail() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.get_bool("section1", "value11");
	}
	#[test]
	fn get_int() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let mut test: HashMap<String, int> = HashMap::new();
		test.insert("integer0".to_string(), 0i);
		test.insert("integer1".to_string(), 1i);
		test.insert("integer2".to_string(), 2i);
		test.insert("integer3".to_string(), 03i);
		for (key, expected) in test.iter() {
			let found = ini.get_int("Integers", key.as_slice());
			assert!((expected*1) == found,
				format!("[Integers] {:?} must be \"{:?}\", not \"{:?}\".", key, expected, found));
		}
	}
	#[test]
	#[should_fail]
	fn get_int_fail() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.get_int("section1", "value11");
	}
	#[test]
	fn get_f64() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let mut test: HashMap<String, f64> = HashMap::new();
		test.insert("float01".to_string(), 0.1f64);
		test.insert("float11".to_string(), 1.1f64);
		test.insert("float20".to_string(), 2.0f64);
		test.insert("float30".to_string(), 3.0f64);
		for (key, expected) in test.iter() {
			let found = ini.get_f64("Floats", key.as_slice());
			assert!((expected*1.0f64) == found,
				format!("[Floats] {:?} must be \"{:?}\", not \"{:?}\".", key, expected, found));
		}
	}
	#[test]
	fn add_section() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let expected = vec!["section1".to_string(),  "section2".to_string(),  "Booleans".to_string(),  "Integers".to_string(),  "Floats".to_string()];
		let found = ini.sections();
		assert!(expected == found, format!("Sections must be \"{:?}\", not {:?}.", expected, found));
		ini.add_section("New section");
		let expected2 = vec!["section1".to_string(),  "section2".to_string(),  "Booleans".to_string(),  "Integers".to_string(),  "Floats".to_string(),  "New section".to_string()];
		let found2 = ini.sections();
		assert!(expected2 == found2, format!("Sections must be \"{:?}\", not {:?}.", expected2, found2));
	}
	#[test]
	#[should_fail]
	fn add_section_twice() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.add_section("New section");
		ini.add_section("New section");
	}
	#[test]
	fn remove_section() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.remove_section("section1".to_string());
		let expected = vec!["section2".to_string(),  "Booleans".to_string(),  "Integers".to_string(),  "Floats".to_string()];
		let found = ini.sections();
		assert!(expected == found, format!("Sections must be \"{:?}\", not {:?}.", expected, found));
	}
	#[test]
	fn set() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.set("section1".to_string(),  "value2".to_string(),  "string 2".to_string());
		let expected = "string 2".to_string();
		let found = ini.get("section1", "value2");
		assert!(expected == found, format!("[section1] value2 must be \"{}\", not \"{}\".", expected, found));
	}
	#[test]
	fn options() {
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let expected = vec!("value11".to_string(),  "value".to_string());
		let found = ini.options("section1".to_string());
		assert!(expected == found, format!("Items of [section1] must be \"{:?}\", not {:?}.", expected, found));
	}
	#[test]
	fn to_string() {
		let filepath = "data/config.ini";
		let path = Path::new(filepath);
		let file = File::open(&path);
		match file {
			Err(e) => panic!("open of {:?} failed: {}", path, e),
			_ => { }
		}
		let mut reader = BufferedReader::new(file);
		let mut lines: Vec<String> = Vec::new();
		for line in reader.lines() {
			match line {
				Ok(nread) => lines.push(nread),
				Err(e) => println!("error reading: {}", e)
			}
		}
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		let found = ini.to_string();
		let expected = lines.concat();
		assert_eq!(expected, found);
	}
	#[test]
	fn write() {
		// Copy config.ini to write_test.ini using `write()`.
		let writepath = "data/write_test.ini";
		let mut ini = super::IniFile::new();
		ini.read("data/config.ini");
		ini.write(writepath);

		// Check that the new file exists
		let path = Path::new(writepath);
		assert!(path.exists(), format!("{} should exist after writing inifile!", writepath));

		// Check the contents
		let mut ini2 = super::IniFile::new();
		ini2.read(writepath);
		let found = ini2.to_string();
		let expected = ini.to_string();
		assert_eq!(expected, found);

		// Clean
		assert!(path.exists(), format!("{} should exist after reading the new inifile!", writepath));
		let result = fs::unlink(&path);
		assert!(!result.is_err(), format!("Unlinking {} should not panic!", writepath));
	}
	#[test]
	fn save() {
		let filepath = "data/save_test.ini".to_string();
		let path = Path::new(filepath);
		if path.exists() {
			println!("The file {:?} should not exist before test::save() is executed!", path);
		}

		let mut ini = super::IniFile::new();
		ini.add_section("section1");
		ini.set("section1".to_string(),  "key1".to_string(),  "value1".to_string());
		ini.set_path(path.clone());
		ini.save();


		let file = File::open(&path);
		match file {
			Err(e) => panic!("open of {:?} failed: {}", path, e),
			_ => { }
		}
		let mut reader = BufferedReader::new(file);
		let mut lines: Vec<String> = Vec::new();
		for line in reader.lines() {
			match line {
				Ok(nread) => lines.push(nread),
				Err(e) => println!("error reading: {}", e)
			}
		}

		let found = lines.concat();
		let expected = "[section1]\nkey1=value1\n".to_string();
		assert_eq!(expected, found);
		match fs::unlink(&path) {
			Err(e) => panic!("open of {:?} failed: {}", path, e),
			_ => { }
		}
	}
}
*/

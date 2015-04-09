
#![allow(unstable)]

use std::os::env;
use std::io::process::Command;
use std::io::fs;
use std::path::Path;
use std::io::fs::PathExtensions;

fn main() {
    let home = get_env("HOME").unwrap();
    let scons = concat_path(home, "bin/scons/bin/scons");
    let cargo_manifest_dir = get_env("CARGO_MANIFEST_DIR").unwrap();

    command(scons.as_slice(), &[]);

    let a_files = list_children(cargo_manifest_dir, |path| -> bool {
        if !path.is_file() {
            return false;
        }

        return match path.extension_str() {
            Some(extension) => { extension == "a" },
            None => false
        };
    });
    let a_files : Vec<&str> = a_files.iter().map(|path| path.filename_str().unwrap()).collect();

    command("mv", concat_slices(a_files.as_slice(), &["target/deps"]).as_slice());
}

fn command(command : &str, args : &[&str]) {
    Command::new(command).args(args).status().unwrap();
}

fn get_env(key : &str) -> Option<String> {
    for &(ref _key, ref value) in env().iter() {
        if _key.as_slice() == key {
            return Some(value.clone());
        }
    }

    return None;
}

fn concat_path(dir : String, child : &str) -> String {
    return format!("{}/{}", dir, child);
}

fn list_children<P>(dir : String, predicate : P) -> Vec<Path>
        where P : Fn(&Path) -> bool {
    let mut children : Vec<Path> = Vec::new();

    let paths = fs::readdir(&Path::new(dir)).unwrap();

    for path in paths.iter() {
        if predicate(path) {
            children.push(path.clone());
        }
    }

    return children;
}

fn concat_slices<'a, T : Clone>(slice1 : &'a [T], slice2 : &'a [T]) -> Vec<T> {
    return [slice1, slice2].concat().clone();
}


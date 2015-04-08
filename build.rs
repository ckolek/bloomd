
#![allow(unstable)]

use std::os::env;
use std::io::process::Command;

fn main() {
    let home = get_env("HOME").unwrap();
    let scons = concat_path(home, "bin/scons/bin/scons");
    let cargo_manifest_dir = get_env("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = get_env("OUT_DIR").unwrap();

    command(scons.as_slice(), &[]);
    command("mv", &[concat_path(cargo_manifest_dir, "*.a").as_slice(), out_dir.as_slice()]);
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

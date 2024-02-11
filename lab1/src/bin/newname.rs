// src/bin/newname.rs
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: newname <old_name> <new_name>");
        return;
    }
    if let Err(e) = fs::rename(&args[1], &args[2]) {
        eprintln!("Error renaming file: {}", e);
    }
}
// src/bin/newname.rs
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: newname <old_name> <new_name>");
        return;
    }
    match fs::rename(&args[1], &args[2]) {
        Ok(()) => println!("Renamed {} to {}", &args[1], &args[2]),
        Err(e) => eprintln!("Failed to rename {}: {}", &args[1], e),
    }
}
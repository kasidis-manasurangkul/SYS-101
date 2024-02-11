// src/bin/destroy.rs
use std::env;
use std::fs;

fn main() {
    for arg in env::args().skip(1) {
        match fs::remove_file(&arg) {
            Ok(()) => println!("Removed {}", &arg),
            Err(e) => eprintln!("Failed to remove {}: {}", arg, e),
        }
    }
}
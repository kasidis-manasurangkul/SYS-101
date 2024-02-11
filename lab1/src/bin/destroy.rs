// src/bin/destroy.rs
use std::env;
use std::fs;

fn main() {
    for arg in env::args().skip(1) {
        if let Err(e) = fs::remove_file(arg) {
            eprintln!("Error deleting file: {}", e);
        }
    }
}
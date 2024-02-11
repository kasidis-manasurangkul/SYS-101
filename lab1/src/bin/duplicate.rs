// src/bin/duplicate.rs
use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: duplicate <source> <destination>");
        return Ok(());
    }
    fs::copy(&args[1], &args[2])?;
    Ok(())
}
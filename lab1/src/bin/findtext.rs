// src/bin/findtext.rs
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: findtext <pattern> <file1> <file2> ...");
        return Ok(());
    }

    let pattern = &args[1];
    for filename in &args[2..] {
        let file = File::open(filename)?;
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if line.contains(pattern) {
                println!("{}: {}", filename, line);
            }
        }
    }
    Ok(())
}

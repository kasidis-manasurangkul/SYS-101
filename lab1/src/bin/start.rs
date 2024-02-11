// src/bin/start.rs
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let mut num_lines = 10;

    if let Some(first_arg) = args.next() {
        if first_arg.starts_with('-') {
            num_lines = first_arg[1..].parse().unwrap_or(10);
        } else {
            args = env::args().skip(2);
            print_file(&first_arg, num_lines)?;
        }
    }

    for arg in args {
        print_file(&arg, num_lines)?;
    }
    Ok(())
}

fn print_file(filename: &str, num_lines: usize) -> io::Result<()> {
    let file = File::open(filename)?;
    for line in io::BufReader::new(file).lines().take(num_lines) {
        println!("{}", line?);
    }
    Ok(())
}
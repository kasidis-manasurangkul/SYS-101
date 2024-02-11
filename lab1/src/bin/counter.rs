// src/bin/counter.rs
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut count_words = true;
    let mut count_lines = true;
    let mut count_chars = true;

    let files = if args.len() > 1 && args[1].starts_with('-') {
        count_words = args[1].contains('w');
        count_lines = args[1].contains('l');
        count_chars = args[1].contains('c');
        &args[2..]
    } else {
        &args[1..]
    };

    for filename in files {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        if count_words {
            let words = content.split_whitespace().count();
            println!("{}: {} words", filename, words);
        }

        if count_lines {
            let lines = content.lines().count();
            println!("{}: {} lines", filename, lines);
        }

        if count_chars {
            let chars = content.chars().count();
            println!("{}: {} characters", filename, chars);
        }
    }
    Ok(())
}

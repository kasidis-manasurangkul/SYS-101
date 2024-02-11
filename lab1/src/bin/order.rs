// src/bin/order.rs
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let mut reverse = false;

    let files = if let Some(first_arg) = args.next() {
        if first_arg == "-r" {
            reverse = true;
            args.collect()
        } else {
            vec![first_arg].into_iter().chain(args).collect()
        }
    } else {
        vec![]
    };

    let mut lines: Vec<String> = vec![];
    for filename in files {
        let file = File::open(filename)?;
        for line in io::BufReader::new(file).lines() {
            lines.push(line?);
        }
    }

    lines.sort();
    if reverse {
        lines.reverse();
    }

    for line in lines {
        println!("{}", line);
    }
    Ok(())
}

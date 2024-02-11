// src/main.rs
mod dir;
mod destroy;
mod newname;
mod duplicate;
mod start;
mod counter;
mod findtext;
mod order;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No command provided");
        return;
    }

    match args[1].as_str() {
        "dir" => dir::execute(),
        "destroy" => destroy::execute(args[2..].to_vec()),
        "newname" => {
            if args.len() < 4 {
                eprintln!("Usage: newname <old_name> <new_name>");
                return;
            }
            newname::execute(&args[2], &args[3]);
        }
        // ... similarly for other commands ...
        _ => eprintln!("Unknown command"),
    }
}

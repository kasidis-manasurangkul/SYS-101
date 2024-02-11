// src/main.rs

use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;


fn main() {
    loop {
        // Display the current working directory as the prompt
        let current_dir = env::current_dir().unwrap();
        print!("{}> ", current_dir.display());
        io::stdout().flush().unwrap(); // Make sure the prompt is written out

        // Read a line of input from the user
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Remove the newline character from the input
        let input = input.trim();

        // Exit the shell on 'exit' command
        if input == "exit" {
            break;
        }

        // Handle the 'cd' command separately
        if input.starts_with("cd") {
            let dir = input.split_whitespace().nth(1).unwrap_or("/");
            if let Err(e) = env::set_current_dir(Path::new(dir)) {
                eprintln!("Error changing directory: {}", e);
            }
            continue;
        }
        

        // Split the input into command and arguments
        let mut parts = input.split_whitespace();
        let command = match parts.next() {
            Some(cmd) => cmd,
            None => continue, // Continue the loop if no command was entered
        };
        let args = parts;

        // Execute the command
        match Command::new(command).args(args).status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("Command returned non-zero exit status {}", status);
                }
            },
            Err(e) => eprintln!("Failed to execute command: {}", e),
        }
    }
}
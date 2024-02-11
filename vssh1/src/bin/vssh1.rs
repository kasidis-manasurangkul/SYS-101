// src/main.rs

use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

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

        // Handle piping commands
        if input.contains("|") {
            let mut commands = input.split("|").peekable();
            let mut previous_command = None;

            while let Some(command) = commands.next() {
                let stdin = previous_command.map_or(
                    std::process::Stdio::inherit(),
                    |output: std::process::Child| std::process::Stdio::from(output.stdout.unwrap()),
                );
                let stdout = if commands.peek().is_some() {
                    // There is another command piped behind this one
                    // Prepare to send output to the next command
                    std::process::Stdio::piped()
                } else {
                    // There are no more commands piped behind this one
                    // Prepare to send output to shell stdout
                    std::process::Stdio::inherit()
                };

                let args: Vec<&str> = command.trim().split_whitespace().collect();
                let command = args[0];
                let args = &args[1..];

                let child = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();

                match child {
                    Ok(child) => {
                        previous_command = Some(child);
                    }
                    Err(e) => {
                        previous_command = None;
                        eprintln!("Error: {}", e);
                    }
                }
            }

            if let Some(mut final_command) = previous_command {
                // Block until the final command has finished
                final_command.wait().unwrap();
            }
            continue;
        } 
        // Handle input redirection
        else if input.contains("<") {
            let mut commands = input.split("<");
            let command = commands.next().unwrap().trim();
            let args: Vec<&str> = command.split_whitespace().collect();
            let file = commands.next().unwrap().trim();
            let file = Path::new(file);

            let output = Command::new(command)
                .args(args[1..].iter())
                .stdin(std::fs::File::open(file).unwrap())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output();

            match output {
                Ok(output) => {
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            continue;
        }
        else {
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
                }
                Err(e) => eprintln!("Failed to execute command: {}", e),
            }
        }
    }
}

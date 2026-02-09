#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;
use std::error::Error;

fn parse_command(command: &str) -> Vec<String> {
    if command == "exit" {
        exit(0);
    }
    let commands: Vec<String> = command.split(' ').map(String::from).collect();
    commands
}

fn execute_command(tokens: Vec<String>) {
    if tokens[0] == "echo" {
        for token in &tokens[1..] {
            print!("{} ", token);
        }
    } else {
        let command = tokens.into_iter().collect::<String>();
        println!("{}: command not found", command);
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        input.clear();
        stdin.read_line(input).unwrap();
        let command = input.trim();
        if !command.is_empty() {
            let parsed_command = parse_command(command);
            execute_command(parsed_command);
        }
        io::stdout().flush().unwrap();
    }
}

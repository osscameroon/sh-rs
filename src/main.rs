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
    let sorted_builtins = vec!["echo", "exit", "type"];
    if tokens[0] == "echo" {
        for token in &tokens[1..tokens.len()-1] {
            print!("{} ", token);
        }
        println!("{}", tokens[tokens.len()-1]);
    } else if tokens[0] == "type" {
        if tokens.len() == 1 {
            return;
        }
        match sorted_builtins.binary_search(&tokens[1].as_str()) {
            Ok(_) => println!("{} is a shell builtin", tokens[1]),
            Err(_) => println!("{}: not found", tokens[1]),
        }
    }
    else {
        // let command = tokens.into_iter().collect::<String>();
        println!("{}: command not found", tokens[0]);
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        io::stdout().flush().unwrap();
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

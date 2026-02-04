#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        input.clear();
        stdin.read_line(input).unwrap();
        let command = input.trim();
        if command == "exit" {
            exit(0);
        }
        if !command.is_empty() {
            println!("{}: command not found", input);
        }
        io::stdout().flush().unwrap();
    }
}

#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let input = &mut String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        input.clear();
        stdin.read_line(input).unwrap();
        let command = input.trim();
        if !command.is_empty() {
            println!("{}: command not found", command);
        }
        io::stdout().flush().unwrap();
    }
}

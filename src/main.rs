#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let mut stdin = io::stdin();
    let input = &mut String::new();
    loop {
        print!("$ ");
        input.clear();
        stdin.read_line(input).unwrap();
        let command = input.trim();
        if !command.is_empty() {
            println!("{}: command not found", input);
        }
        io::stdout().flush().unwrap();
    }
}

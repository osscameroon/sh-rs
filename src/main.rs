use std::io::{self, Write};
use std::process::Command;

fn main() {
    loop {
        let mut command = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).expect("Error reading user input");
        let args: Vec<&str> = command[..command.len()-1].split(" ").collect();
        let output = Command::new(args[0]).args(&args[1..]).output().expect("Command execution failed");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}

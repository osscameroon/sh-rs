#[allow(unused_imports)]
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::fs::File;
use std::process::{Command, exit};
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

fn parse_command(command: &str) -> Vec<String> {
    if command == "exit" {
        exit(0);
    }
    // Should trim the splitted strings otherwise nasty stuff can happen
    let commands: Vec<String> = command.split(' ').filter(|s| *s != "").map(|s| String::from(s)).collect();
    commands
}

fn parse_environment_path() -> Vec<PathBuf> {
    match env::var_os("PATH") {
        Some(paths) => {
            let mut sanitized_path: Vec<PathBuf> = vec![];
            for path in env::split_paths(&paths) {
                if path.is_dir() {
                    sanitized_path.push(path)
                }
            }
            sanitized_path
        }
        None => { println!("PATH not defined in the environment"); vec![PathBuf::new()] },
    }

}

fn search_environment_path(sanitized_environment_path: Vec<PathBuf>, command: String) -> Result<PathBuf, Box<dyn Error>> {
    for path in sanitized_environment_path {
        let full_path: PathBuf = path.join(&command);
        if full_path.is_file() {
            let file = File::open(&full_path)?;
            let mode = file.metadata()?.permissions().mode();
            let executable_mode = 0o100;
            if (mode & executable_mode) == executable_mode {
                return Ok(full_path)
            }
        }
    }
    Err("command not found in path".into())
}

fn execute_command(tokens: Vec<String>) {
    let sorted_builtins = vec!["echo", "exit", "type"];
    let sanitized_environment_path = parse_environment_path();
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
            Err(_) => match search_environment_path(sanitized_environment_path, tokens[1].clone()) {
                Ok(executable_path) => println!("{} is {}", tokens[1], executable_path.display()),
                Err(_) => println!("{}: not found", tokens[1]),
            },
        }
    }
    else {
        match search_environment_path(sanitized_environment_path, tokens[0].clone()) {
            Ok(executable_path) => { Command::new(tokens[0].clone())
                .args(&tokens[1..])
                .status()
                .expect("Failed to execute command");},
            Err(_) => println!("{}: command not found", tokens[0]),
        }
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

#[allow(unused_imports)]
use regex::Regex;
use rustix::process::chdir;
use rustix::path::Arg;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::fs::File;
use std::process::{Command, exit};
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    OutsideSingleQuotes,
    InsideSingleQuotes,
    OutsideDoubleQuotes,
    InsideDoubleQuotes,
}

fn change_directory<P: Arg>(absolute_path: P) -> bool {
    match chdir(absolute_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut state = match input.find('"') {
        Some(pos_single_quote) => match input.find('\'') {
            Some(pos_double_quote) =>
            {
                if pos_single_quote < pos_double_quote {
                    State::OutsideSingleQuotes
                } else {
                    State::OutsideDoubleQuotes
                }
            },
            None => State::OutsideDoubleQuotes,
        },
        None => State::OutsideSingleQuotes,
    };
    let mut cursor = input.chars();
    let mut buffer = String::from("");
    let mut tokens = vec![];
    while let Some(c) = cursor.next() {
        // I wonder at which point I will have to write an actual parser
        // This doesn't support being inside DoubleQuotes and inside Single Quotes
        // $ echo "hello 'me' is"
        // "hello me is"
        match (state, c) {
            (State::InsideDoubleQuotes, c) if c.is_whitespace() => {
                buffer.push(c);
            }
            (State::OutsideDoubleQuotes, '"') => {
                state = State::InsideDoubleQuotes;
                if !buffer.is_empty() {
                    tokens.push(buffer.clone());
                    buffer.clear();
                }
            }
            (State::InsideDoubleQuotes, '"') => {
                state = State::OutsideDoubleQuotes;
                tokens.push(buffer.clone());
                buffer.clear();
            }
            (State::OutsideDoubleQuotes, c) if c.is_whitespace() => {
                if buffer.trim().is_empty() {
                    continue
                }
                tokens.push(String::from(buffer.trim()));
                buffer.clear();
            },
            (State::OutsideSingleQuotes, '\'') => {
                state = State::InsideSingleQuotes;
                if !buffer.is_empty() {
                    tokens.push(buffer.clone());
                    buffer.clear();
                }
            },
            (State::OutsideSingleQuotes, c) if c.is_whitespace() => {
                if buffer.trim().is_empty() {
                    continue
                }
                tokens.push(String::from(buffer.trim()));
                buffer.clear();
            },
            (State::OutsideSingleQuotes, c) if c.is_ascii() => {
                buffer.push(c);
            },
            (State::InsideSingleQuotes, '\'') => {
                state = State::OutsideSingleQuotes;
                tokens.push(buffer.clone());
                buffer.clear();
            },
            (State::InsideSingleQuotes, c) if c.is_ascii() => {
                buffer.push(c);
            },
            _ => { buffer.push(c); },
        }
    }
    if !buffer.is_empty() {
        tokens.push(buffer.clone());
    }
    tokens
}

fn parse_command(command: &str) -> Vec<String> {
    let re = Regex::new(r"(?<command_name>\s*\S+\s*)(?<arguments>.*)").unwrap();
    let Some(capture) = re.captures(command) else {
        println!("No match");
        return vec![String::from("")];
    };
    let mut commands: Vec<String> = vec![String::from(capture["command_name"].trim())];
    if !capture["arguments"].is_empty() {
        let arguments = capture["arguments"].replace("''", "");
        commands.extend(tokenize(&arguments.replace("\"\"", "").as_str()));
    }
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
    let sorted_builtins = vec!["cd", "echo", "exit", "pwd", "type"];
    let sanitized_environment_path = parse_environment_path();
    if tokens[0] == "exit" {
        exit(0);
    }
    if tokens[0] == "echo" {
        for token in &tokens[1..tokens.len()-1] {
            print!("{} ", token);
        }
        println!("{}", tokens[tokens.len()-1]);
    } else if tokens[0] == "cd" {
        if tokens.len() != 2 {
            return;
        }
        if tokens[1] == "~" {
            match env::home_dir() {
                Some(path) => {
                    if !change_directory(&path) {
                        println!("{}: No such file or directory", path.display());
                    }
                    return;
                },
                None => println!("Impossible to get your home dir!"),
            }
        }
        if !change_directory(&tokens[1]) {
            println!("{}: No such file or directory", tokens[1]);
        }
    } else if tokens[0] == "type" {
        if tokens.len() == 1 {
            return;
        }
        match sorted_builtins.binary_search(&tokens[1].as_str()) {
            Ok(_) => println!("{} is a shell builtin", tokens[1]),
            Err(_) => match search_environment_path(sanitized_environment_path, tokens[1].clone()) {
                Ok(executable_path) => {
                    let executable_path: PathBuf = executable_path;
                    println!("{} is {}", tokens[1], executable_path.display());
                },
                Err(_) => println!("{}: not found", tokens[1]),
            },
        }
    } else if tokens[0] == "pwd" {
        match env::current_dir() {
            Ok(current_dir) => println!("{}", current_dir.display()),
            Err(_) => panic!("Cannot determine current dir"),
        }
    } else {
        match search_environment_path(sanitized_environment_path, tokens[0].clone()) {
            Ok(_) => { Command::new(tokens[0].clone())
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

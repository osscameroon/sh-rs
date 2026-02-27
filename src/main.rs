#[allow(unused_imports)]
use regex::Regex;
use rustix::path::Arg;
use rustix::process::chdir;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, exit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    OutsideSingleQuotes,
    InsideSingleQuotes,
    OutsideDoubleQuotes,
    InsideDoubleQuotes,
    Redirect,
    NoRedirect,
}

fn change_directory<P: Arg>(absolute_path: P) -> bool {
    match chdir(absolute_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn tokenize(input: &str) -> (Vec<String>, String) {
    let mut state = match input.find('"') {
        Some(pos_double_quote) => match input.find('\'') {
            Some(pos_single_quote) => {
                if pos_single_quote < pos_double_quote {
                    State::OutsideSingleQuotes
                } else {
                    State::OutsideDoubleQuotes
                }
            }
            None => State::OutsideDoubleQuotes,
        },
        None => State::OutsideSingleQuotes,
    };
    let mut cursor = input.chars();
    let mut buffer = String::from("");
    let mut tokens = vec![];
    let mut redirect_state = State::NoRedirect;
    let mut output_file = String::from("");
    while let Some(c) = cursor.next() {
        match (state, c) {
            (State::InsideSingleQuotes, '\\') => {
                buffer.push(c);
            }
            (_, '\\') => {
                match cursor.next() {
                    Some(c) => buffer.push(c),
                    None => break,
                };
            }
            (State::InsideDoubleQuotes, c) if c.is_whitespace() => {
                buffer.push(c);
            }
            (State::OutsideDoubleQuotes, '"') => {
                state = State::InsideDoubleQuotes;
                if !buffer.is_empty() {
                    if buffer.as_str().ends_with(|c: char| c.is_whitespace()) {
                        if redirect_state == State::Redirect {
                            redirect_state = State::NoRedirect;
                            output_file = buffer.clone();
                        } else {
                            tokens.push(buffer.clone());
                        }
                        buffer.clear();
                    } else {
                        continue;
                    }
                }
            }
            (State::InsideDoubleQuotes, '"') => {
                state = State::OutsideDoubleQuotes;
            }
            (State::OutsideDoubleQuotes, c) if c.is_whitespace() => {
                if buffer.trim().is_empty() {
                    continue;
                }
                if redirect_state == State::Redirect {
                    redirect_state = State::NoRedirect;
                    output_file = String::from(buffer.trim());
                } else {
                    tokens.push(String::from(buffer.trim()));
                }
                buffer.clear();
            }
            (State::OutsideSingleQuotes, '\'') => {
                state = State::InsideSingleQuotes;
                if !buffer.is_empty() {
                    if buffer.as_str().ends_with(|c: char| c.is_whitespace()) {
                        if redirect_state == State::Redirect {
                            redirect_state = State::NoRedirect;
                            output_file = buffer.clone();
                        } else {
                            tokens.push(buffer.clone());
                        }
                        buffer.clear();
                    } else {
                        continue;
                    }
                }
            }
            (State::OutsideSingleQuotes, c) if c.is_whitespace() => {
                if buffer.trim().is_empty() {
                    continue;
                }
                if redirect_state == State::Redirect {
                    redirect_state = State::NoRedirect;
                    output_file = String::from(buffer.trim());
                } else {
                    tokens.push(String::from(buffer.trim()));
                }
                buffer.clear();
            }
            (_, '1') => {
                match cursor.next() {
                    Some(c) => {
                        if c == '>' {
                            redirect_state = State::Redirect;
                        } else {
                            buffer.push('1');
                            buffer.push(c);
                        }
                    }
                    None => buffer.push('1'),
                };
            }
            (_, '>') => {
                redirect_state = State::Redirect;
            }
            (State::OutsideSingleQuotes, c) if c.is_ascii() => {
                buffer.push(c);
            }
            (State::InsideSingleQuotes, '\'') => {
                state = State::OutsideSingleQuotes;
                tokens.push(buffer.clone());
                buffer.clear();
            }
            (State::InsideSingleQuotes, c) if c.is_ascii() => {
                buffer.push(c);
            }
            _ => {
                buffer.push(c);
            }
        }
    }
    if !buffer.is_empty() {
        if redirect_state == State::Redirect {
            output_file = buffer.clone();
        } else {
            tokens.push(buffer.clone());
        }
    }
    (tokens, output_file)
}

fn parse_command(command: &str) -> (Vec<String>, String) {
    let mut commands: Vec<String> = vec![];
    if !command.is_empty() {
        let (arguments, output_file) =
            tokenize(&command.replace("''", "").replace("\"\"", "").as_str());
        //println!("Args: {:?}, Output: {}", arguments, output_file);
        commands.extend(arguments);
        return (commands, output_file);
    }
    (commands, String::from(""))
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
        None => {
            println!("PATH not defined in the environment");
            vec![PathBuf::new()]
        }
    }
}

fn search_environment_path(
    sanitized_environment_path: Vec<PathBuf>,
    command: String,
) -> Result<PathBuf, Box<dyn Error>> {
    for path in sanitized_environment_path {
        let full_path: PathBuf = path.join(&command);
        if full_path.is_file() {
            let file = File::open(&full_path)?;
            let mode = file.metadata()?.permissions().mode();
            let executable_mode = 0o100;
            if (mode & executable_mode) == executable_mode {
                return Ok(full_path);
            }
        }
    }
    Err("command not found in path".into())
}

fn make_writer(dest: String) -> io::Result<Box<dyn Write>> {
    if dest == "" {
        return Ok(Box::new(io::stdout().lock()));
    } else {
        return Ok(Box::new(File::create(dest)?));
    }
}

fn execute_command(
    tokens: Vec<String>,
    output_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let sorted_builtins = vec!["cd", "echo", "exit", "pwd", "type"];
    let sanitized_environment_path = parse_environment_path();
    let mut writer = make_writer(output_file)?;
    if tokens[0] == "exit" {
        exit(0);
    }
    if tokens[0] == "echo" {
        for token in &tokens[1..tokens.len() - 1] {
            writer.write_all(format!("{} ", token).as_bytes())?;
        }
        writer.write_all(format!("{}\n", tokens[tokens.len() - 1]).as_bytes())?;
    } else if tokens[0] == "cd" {
        if tokens.len() > 2 {
            eprintln!("cd: too many arguments");
        }
        if tokens[1] == "~" {
            match env::home_dir() {
                Some(path) => {
                    if !change_directory(&path) {
                        writer.write_all(
                            format!("{}: No such file or directory", path.display()).as_bytes(),
                        )?;
                    }
                }
                None => eprintln!("cd: Impossible to get your home dir!"),
            }
        }
        if !change_directory(&tokens[1]) {
            eprintln!("{}: No such file or directory", tokens[1]);
        }
    } else if tokens[0] == "type" {
        if tokens.len() == 1 {
            return Ok(());
        }
        match sorted_builtins.binary_search(&tokens[1].as_str()) {
            Ok(_) => writer.write_all(format!("{} is a shell builtin", tokens[1]).as_bytes())?,
            Err(_) => {
                match search_environment_path(sanitized_environment_path, tokens[1].clone()) {
                    Ok(executable_path) => {
                        let executable_path: PathBuf = executable_path;
                        writer.write_all(
                            format!("{} is {}", tokens[1], executable_path.display()).as_bytes(),
                        )?;
                    }
                    Err(_) => eprintln!("{}: not found", tokens[1]),
                }
            }
        };
    } else if tokens[0] == "pwd" {
        match env::current_dir() {
            Ok(current_dir) => writer.write_all(format!("{}", current_dir.display()).as_bytes())?,
            Err(_) => panic!("Cannot determine current dir"),
        }
    } else {
        match search_environment_path(sanitized_environment_path, tokens[0].clone()) {
            Ok(_) => {
                writer.write_all(
                    Command::new(tokens[0].clone())
                        .args(&tokens[1..])
                        .output()
                        .expect("Failed to execute command")
                        .stdout
                        .as_slice(),
                )?;
            }
            Err(_) => eprintln!("{}: command not found", tokens[0]),
        }
    }
    Ok(())
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
            let (parsed_command, output_file) = parse_command(command);
            execute_command(parsed_command, output_file)?;
        }
        io::stdout().flush().unwrap();
    }
}

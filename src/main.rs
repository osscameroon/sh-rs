#[allow(unused_imports)]
use regex::Regex;
use rustix::path::Arg;
use rustix::process::chdir;
use std::env;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuoteState {
    Unquoted,
    SingleQuoted,
    DoubleQuoted,
}

fn change_directory<P: Arg>(absolute_path: P) -> bool {
    match chdir(absolute_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn tokenize(input: &str) -> (Vec<String>, String) {
    let mut quote_state = QuoteState::Unquoted;
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    let mut tokens: Vec<String> = vec![];
    let mut in_token = false;
    let mut redirect = false;
    let mut output_file = String::new();

    while let Some(c) = chars.next() {
        match quote_state {
            QuoteState::Unquoted => match c {
                '\'' => {
                    quote_state = QuoteState::SingleQuoted;
                    in_token = true; // even empty quotes produce a token part
                }
                '"' => {
                    quote_state = QuoteState::DoubleQuoted;
                    in_token = true;
                }
                '\\' => {
                    // backslash escapes the next character
                    if let Some(next) = chars.next() {
                        buffer.push(next);
                        in_token = true;
                    }
                }
                c if c.is_whitespace() => {
                    if in_token {
                        if redirect {
                            redirect = false;
                            output_file = buffer.clone();
                        } else {
                            tokens.push(buffer.clone());
                        }
                        buffer.clear();
                        in_token = false;
                    }
                }
                '>' => {
                    // flush current token if any
                    if in_token {
                        tokens.push(buffer.clone());
                        buffer.clear();
                        in_token = false;
                    }
                    redirect = true;
                }
                '1' => {
                    // check for 1> redirect (only if not already in a token)
                    if !in_token {
                        if chars.peek() == Some(&'>') {
                            chars.next(); // consume '>'
                            redirect = true;
                        } else {
                            buffer.push(c);
                            in_token = true;
                        }
                    } else {
                        buffer.push(c);
                    }
                }
                _ => {
                    buffer.push(c);
                    in_token = true;
                }
            },
            QuoteState::SingleQuoted => match c {
                '\'' => {
                    // close single quote — everything inside is literal
                    quote_state = QuoteState::Unquoted;
                }
                _ => {
                    buffer.push(c);
                }
            },
            QuoteState::DoubleQuoted => match c {
                '"' => {
                    // close double quote
                    quote_state = QuoteState::Unquoted;
                }
                '\\' => {
                    // inside double quotes, backslash only escapes: $ ` " \ newline
                    if let Some(&next) = chars.peek() {
                        match next {
                            '$' | '`' | '"' | '\\' | '\n' => {
                                chars.next();
                                buffer.push(next);
                            }
                            _ => {
                                // backslash is literal
                                buffer.push('\\');
                            }
                        }
                    } else {
                        buffer.push('\\');
                    }
                }
                _ => {
                    buffer.push(c);
                }
            },
        }
    }

    // flush remaining buffer
    if in_token || !buffer.is_empty() {
        if redirect {
            output_file = buffer;
        } else {
            tokens.push(buffer);
        }
    }

    (tokens, output_file)
}

fn parse_command(command: &str) -> (Vec<String>, String) {
    let mut commands: Vec<String> = vec![];
    if !command.is_empty() {
        let (arguments, output_file) = tokenize(command);
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
            eprintln!("PATH not defined in the environment");
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
        let path = Path::new(dest.as_str());
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
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
            Ok(_) => writer.write_all(format!("{} is a shell builtin\n", tokens[1]).as_bytes())?,
            Err(_) => {
                match search_environment_path(sanitized_environment_path, tokens[1].clone()) {
                    Ok(executable_path) => {
                        let executable_path: PathBuf = executable_path;
                        writer.write_all(
                            format!("{} is {}\n", tokens[1], executable_path.display()).as_bytes(),
                        )?;
                    }
                    Err(_) => eprintln!("{}: not found", tokens[1]),
                }
            }
        };
    } else if tokens[0] == "pwd" {
        match env::current_dir() {
            Ok(current_dir) => {
                writer.write_all(format!("{}\n", current_dir.display()).as_bytes())?
            }
            Err(_) => panic!("Cannot determine current dir"),
        }
    } else {
        match search_environment_path(sanitized_environment_path, tokens[0].clone()) {
            Ok(_) => {
                let output = Command::new(tokens[0].clone())
                    .args(&tokens[1..])
                    .output()
                    .expect("Failed to execute command");
                writer.write_all(output.stdout.as_slice())?;
                if !output.stderr.is_empty() {
                    eprint!("{}", output.stderr.as_str()?);
                };
            }
            Err(_) => eprint!("{}: command not found\n", tokens[0]),
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

use std::io::{self, Write};
use std::process::Command;

#[derive(Debug)]
struct Token {
    cmd: Option<String>,
    params: Vec<String>,
    status: Option<bool>,
    on_read: Option<Box<Token>>,
    on_success: Option<Box<Token>>,
    on_failure: Option<Box<Token>>,
    next: Option<Box<Token>>,
    stdin: Option<String>,
    stdout: Option<String>
}

impl Token {
    fn new() -> Self {
        Self {
            cmd: None,
            params: Vec::new(),
            status: None,
            on_read: None,
            on_success: None,
            on_failure: None,
            next: None,
            stdin: None,
            stdout: None
        }
    }
}

#[derive(Debug)]
struct Cursor {
    word: String,
    prev_char: char
}

impl Cursor {
    fn new() -> Self {
        Self {
            word: String::new(),
            prev_char: '\0'
        }
    }
}

fn parse(s: &str) {
    let mut tokens: Vec<Token> = Vec::new();
    let mut curr_token = Token::new();
    let mut cursor = Cursor::new();

    for c in s.chars() {
        match c {
            ' ' | ';' => {
                match cursor.prev_char {
                    '|' | '&' | '<' | '>' | ' ' | ';' | '\0' => { },
                    _ => {
                        if cursor.word == "" {
                            // Do nothing!
                        } else if curr_token.cmd == None {
                            curr_token.cmd = Some(cursor.word);
                        } else if curr_token.cmd == Some("stdin".to_owned()) {
                            let prev_token = curr_token;
                            curr_token = *prev_token.next.unwrap();
                            curr_token.stdin = Some(cursor.word);
                        } else if curr_token.cmd == Some("stdout".to_owned()) {
                            let prev_token = curr_token;
                            curr_token = *prev_token.next.unwrap();
                            curr_token.stdout = Some(cursor.word);
                        } else {
                            curr_token.params.push(cursor.word);
                        }

                        if c == ';' {
                            tokens.push(curr_token);
                            curr_token = Token::new();
                        }
                    },
                };
                cursor.word = String::new();
            },
            '|' => {
                match cursor.prev_char {
                    '|' => {
                        curr_token.on_failure = curr_token.on_read;
                        curr_token.on_read = None;
                    },
                    _ => {
                        let prev_token = curr_token;
                        curr_token = Token::new();
                        curr_token.on_read = Some(Box::new(prev_token));
                    }
                };
                cursor.word = String::new();
            },
            '&' => {
                match cursor.prev_char {
                    '&' => {
                        curr_token.on_success = curr_token.next;
                        curr_token.next = None;
                    },
                    _ => {
                        let prev_token = curr_token;
                        curr_token = Token::new();
                        curr_token.next = Some(Box::new(prev_token));
                    }
                };
                cursor.word = String::new();
            },
            '>' | '<' => {
                let prev_token = curr_token;
                curr_token = Token::new();
                curr_token.cmd = Some({ if c == '>' { "stdin" } else { "stdout" } }.to_owned());
                curr_token.next = Some(Box::new(prev_token));
            },
            '\n' => {
                tokens.push(curr_token);
                // We suppose end of the commandline
                break;
            },
            _ => {
                cursor.word.push(c);
            }
        }
        println!("{:#?}", cursor);
        cursor.prev_char = c;
    }

    println!("{:#?}", tokens);
}

fn main() {
    loop {
        let mut command = String::new();
        let pwd = Command::new("pwd").output().expect("Getting of the current dir failed");
        print!("{}> ", String::from_utf8_lossy(&pwd.stdout[..pwd.stdout.len()-1]));
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).expect("Error reading user input");
        parse(&command);
    }
}

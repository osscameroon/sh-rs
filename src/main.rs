use std::io::{self, Write};

fn main() {
    loop {
        let mut command = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).expect("Error reading user input");
    }
}

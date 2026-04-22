#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        let mut token_iter = input.split_whitespace().into_iter();
        let command = token_iter.next().unwrap();
        match command {
            "exit" => break,
            "echo" => {
                println!("{}", token_iter.map(|s| s.to_string()).collect::<Vec<String>>().join(" "))
            },
            _ => println!("{}: command not found", command)
        }
    }
    
}

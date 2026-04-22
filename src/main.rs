#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        let mut token_iter = input.split_whitespace().map(|s| s.to_string());
        let command = token_iter.next().unwrap();
        match command.as_str() {
            "exit" => break,
            "echo" => {
                println!("{}", token_iter.collect::<Vec<String>>().join(" "))
            },
            "type" => {
                let snd_command = token_iter.next().unwrap();
                match snd_command.as_str() {
                    "echo" => println!("echo is a shell builtin"),
                    "exit" => println!("exit is a shell builtin"),
                    "type" => println!("type is a shell builtin"),
                    _ => println!("{snd_command}: not found")
                }
            },
            _ => println!("{}: command not found", command)
        }
    }
    
}

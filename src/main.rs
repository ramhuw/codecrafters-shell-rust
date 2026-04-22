use is_executable::IsExecutable;
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

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
            }
            "type" => {
                let snd_command = token_iter.next().unwrap();
                match snd_command.as_str() {
                    "echo" | "exit" | "type" | "pwd" | "cd" => {
                        println!("{} is a shell builtin", snd_command)
                    }
                    _ => match find_executable(&snd_command) {
                        Some(target_path) => {
                            println!("{} is {}", snd_command, target_path.to_str().unwrap())
                        }
                        None => println!("{snd_command}: not found"),
                    },
                }
            }
            "pwd" => {
                println!("{}", env::current_dir().unwrap().to_str().unwrap())
            }
            "cd" => {
                let arg = token_iter.next().unwrap();
                if arg.starts_with("/") {
                    match env::set_current_dir(&arg) {
                        Ok(_) => {}
                        Err(_) => println!("{}: No such file or directory", arg),
                    }
                }
            }
            _ => {
                if let Some(_) = find_executable(&command) {
                    let _ = Command::new(&command).args(token_iter).status();
                } else {
                    println!("{}: command not found", command)
                }
            }
        }
    }
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    for path in env::split_paths(&env::var("PATH").unwrap()) {
        for entry in path.read_dir().unwrap() {
            let valid_entry = entry.unwrap();
            let valid_path = valid_entry.path();
            if valid_path.file_name().and_then(|s| s.to_str()) == Some(cmd)
                && valid_path.is_executable()
            {
                return Some(valid_path);
            }
        }
    }

    None
}

use is_executable::IsExecutable;
use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        let mut token_iter = tokenizer(&input).into_iter();
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
                cd(&arg);
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

fn cd(arg: &str) {
    let home = env::var("HOME").unwrap();
    let home_path = Path::new(&home);
    if arg == "~" {
        env::set_current_dir(home_path).unwrap();
        return;
    } else if arg.starts_with("~/") {
        match env::set_current_dir(home_path.join(&arg[2..])) {
            Ok(_) => {}
            Err(_) => println!("{}: No such file or directory", arg),
        }
        return;
    }
    match env::set_current_dir(&arg) {
        Ok(_) => {}
        Err(_) => println!("{}: No such file or directory", arg),
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

fn tokenizer(input: &String) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for c in input.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' if !in_single && !in_double => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

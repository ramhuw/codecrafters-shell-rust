use is_executable::IsExecutable;
use std::env;
use std::fs::File;
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
        let mut output = String::new();
        let stout = [">", "1>"];
        match command.as_str() {
            "exit" => break,
            "echo" => {
                output = format!(
                    "{}",
                    token_iter
                        .clone()
                        .take_while(|s| s != ">" && s != "1>")
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                token_iter = token_iter
                    .skip_while(|s| ![">", "1>"].contains(&s.as_str()))
                    .collect::<Vec<String>>()
                    .into_iter();
            }
            "type" => {
                let snd_command = token_iter.next().unwrap();
                match snd_command.as_str() {
                    "echo" | "exit" | "type" | "pwd" | "cd" => {
                        output = format!("{} is a shell builtin", snd_command);
                    }
                    _ => match find_executable(&snd_command) {
                        Some(target_path) => {
                            output =
                                format!("{} is {}", snd_command, target_path.to_str().unwrap());
                        }
                        None => output = format!("{snd_command}: not found"),
                    },
                }
            }
            "pwd" => {
                output = format!("{}", env::current_dir().unwrap().to_str().unwrap());
            }
            "cd" => {
                let arg = token_iter.next().unwrap_or("~".to_string());
                cd(&arg);
            }
            _ => {
                if let Some(_) = find_executable(&command) {
                    let out = Command::new(&command)
                        .args(
                            token_iter
                                .clone()
                                .take_while(|s| !stout.contains(&s.as_str())),
                        )
                        .output()
                        .unwrap();

                    output = out
                        .stdout
                        .iter()
                        .map(|u| (*u as char).to_string())
                        .collect::<Vec<String>>()
                        .join("");
                    output.push_str(
                        out.stderr
                            .iter()
                            .map(|u| (*u as char).to_string())
                            .collect::<Vec<String>>()
                            .join("")
                            .as_str(),
                    );
                    token_iter = token_iter
                        .skip_while(|s| !stout.contains(&s.as_str()))
                        .collect::<Vec<String>>()
                        .into_iter();
                } else {
                    println!("{}: command not found", command)
                }
            }
        }
        if !output.is_empty() {
            if let Some(token) = token_iter.next() {
                match token {
                    token if stout.contains(&token.as_str()) => {
                        match File::create(token_iter.next().unwrap()) {
                            Ok(mut file) => {
                                file.write(output.trim().as_bytes()).unwrap();
                            }
                            Err(e) => println!("{}", e),
                        }
                    }
                    _ => {}
                }
            } else {
                println!("{}", output.trim().to_string());
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
    let mut behind_slash = false;
    for c in input.chars() {
        match c {
            '\\' if !in_single && !behind_slash => behind_slash = true,
            '\'' if !in_double && !behind_slash => in_single = !in_single,
            '"' if !in_single && !behind_slash => in_double = !in_double,
            ' ' if !in_single && !in_double && !behind_slash => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                if in_double && behind_slash && !['"', '\\', '$', '`', '\n'].contains(&c) {
                    current.push('\\');
                }
                behind_slash = false;
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

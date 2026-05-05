use is_executable::IsExecutable;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

enum Builtin {
    Exit,
    Echo,
    Type,
    PWD,
    CD,
    Command { command: String },
}

impl Builtin {
    fn from(token: &str) -> Self {
        match token {
            "exit" => Self::Exit,
            "echo" => Self::Echo,
            "type" => Self::Type,
            "pwd" => Self::PWD,
            "cd" => Self::CD,
            command => Self::Command {
                command: command.to_string(),
            },
        }
    }
}

impl ToString for Builtin {
    fn to_string(&self) -> String {
        match self {
            Self::Exit => String::from("exit"),
            Self::Echo => String::from("echo"),
            Self::Type => String::from("type"),
            Self::PWD => String::from("pwd"),
            Self::CD => String::from("cd"),
            Self::Command { command } => String::from(command),
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().lock().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
        let mut token_iter = tokenizer(input.trim()).into_iter();
        let redirect_stdout = [">", "1>"];
        let redirects = [">", "1>"];
        if let Some(command) = token_iter.next() {
            let args = token_iter
                .clone()
                .take_while(|s| !redirects.contains(&s.as_str()));
            let mut stdout_iter = token_iter.skip_while(|s| !redirects.contains(&s.as_str()));
            let builtin = Builtin::from(&command);
            let mut stdout = match builtin {
                Builtin::Exit => break,
                Builtin::Echo => handle_echo(args),
                Builtin::Type => handle_type(args),
                Builtin::PWD => format!("{}", env::current_dir().unwrap().to_str().unwrap()),
                Builtin::CD => handle_cd(args),
                Builtin::Command { command } => {
                    if let Some(_) = find_executable(&command) {
                        let output = Command::new(&command).args(args).output().unwrap();
                        std::io::stderr().write_all(&output.stderr).unwrap();
                        String::from_utf8(output.stdout).unwrap().trim().to_string()
                    } else {
                        String::new()
                    }
                }
            };
            stdout = stdout.trim().to_string();
            if let Some(redirect) = stdout_iter.next() {
                if redirect_stdout.contains(&redirect.as_str()) {
                    let mut file = File::create(stdout_iter.next().unwrap()).unwrap();
                    file.write_all(stdout.as_bytes()).unwrap();
                }
            } else if !stdout.is_empty() {
                println!("{}", stdout);
            }
        }
    }
}

fn handle_echo(args: impl Iterator<Item = String>) -> String {
    args.collect::<Vec<String>>().join(" ")
}

fn handle_type(args: impl Iterator<Item = String>) -> String {
    args.map(|arg| find_type(arg))
        .collect::<Vec<String>>()
        .join("\n")
}

fn find_type(arg: String) -> String {
    match Builtin::from(arg.as_str()) {
        Builtin::Command { command } => match find_executable(&command) {
            Some(target_path) => format!("{} is {}", arg, target_path.to_str().unwrap()),
            None => format!("{arg}: not found"),
        },
        _ => format!("{} is a shell builtin", arg),
    }
}

fn handle_cd(args: impl Iterator<Item = String>) -> String {
    let mut args = args;
    let arg = args.next().unwrap_or("~".to_string());
    if let Some(_) = args.next() {
        eprintln!("cd: too many arguments");
    }
    let home = env::var("HOME").unwrap();
    let home_path = Path::new(&home);
    if arg == "~" {
        env::set_current_dir(home_path).unwrap();
    } else if arg.starts_with("~/") {
        match env::set_current_dir(home_path.join(&arg[2..])) {
            Ok(_) => {}
            Err(_) => eprintln!("{}: No such file or directory", arg),
        }
    } else {
        match env::set_current_dir(&arg) {
            Ok(_) => {}
            Err(_) => eprintln!("{}: No such file or directory", arg),
        }
    }
    String::new()
}

fn find_executable(command: &str) -> Option<PathBuf> {
    for path in env::split_paths(&env::var("PATH").unwrap()) {
        for entry in path.read_dir().unwrap() {
            let valid_entry = entry.unwrap();
            let valid_path = valid_entry.path();
            if valid_path.file_name().and_then(|s| s.to_str()) == Some(command)
                && valid_path.is_executable()
            {
                return Some(valid_path);
            }
        }
    }

    None
}

fn tokenizer(input: &str) -> Vec<String> {
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

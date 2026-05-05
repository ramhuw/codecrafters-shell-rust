use is_executable::IsExecutable;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Output {
    stdout: String,
    stderr: String,
}

impl Add for Output {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            stdout: self.stdout + &other.stdout,
            stderr: self.stderr + &other.stderr,
        }
    }
}

impl Output {
    fn new() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
        }
    }
    fn from_stdout(s: String) -> Self {
        Self {
            stdout: s,
            stderr: String::new(),
        }
    }

    fn from_stderr(s: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: s,
        }
    }
}

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
        let redirect_stderr = ["2>"];
        let append_stdout = [">>", "1>>"];
        let append_stderr = ["2>>"];
        let stops = [">", "1>", "2>", ">>", "1>>", "2>>"];
        if let Some(command) = token_iter.next() {
            let args = token_iter
                .clone()
                .take_while(|s| !stops.contains(&s.as_str()));
            let mut stdout_iter = token_iter.skip_while(|s| !stops.contains(&s.as_str()));
            let builtin = Builtin::from(&command);
            let output: Output = match builtin {
                Builtin::Exit => break,
                Builtin::Echo => handle_echo(args),
                Builtin::Type => handle_type(args),
                Builtin::PWD => Output::from_stdout(format!(
                    "{}",
                    env::current_dir().unwrap().to_str().unwrap()
                )),
                Builtin::CD => handle_cd(args),
                Builtin::Command { command } => {
                    if !find_executable(&command).is_none() {
                        let output = Command::new(&command).args(args).output().unwrap();
                        Output {
                            stdout: String::from_utf8(output.stdout).unwrap().trim().to_string(),
                            stderr: String::from_utf8(output.stderr).unwrap().trim().to_string(),
                        }
                    } else {
                        Output::from_stderr(format!("{}: command not found", command))
                    }
                }
            };
            if let Some(stop) = stdout_iter.next() {
                let file_name = stdout_iter.next().unwrap();
                if redirect_stdout.contains(&stop.as_str()) {
                    let mut file = File::create(file_name).unwrap();
                    file.write_all(output.stdout.as_bytes()).unwrap();
                    if !output.stderr.is_empty() {
                        println!("{}", output.stderr);
                    }
                } else if redirect_stderr.contains(&stop.as_str()) {
                    let mut file = File::create(file_name).unwrap();
                    file.write_all(output.stderr.as_bytes()).unwrap();
                    if !output.stdout.is_empty() {
                        println!("{}", output.stdout);
                    }
                } else if append_stdout.contains(&stop.as_str()) {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(file_name)
                        .unwrap();
                    file.write_all(output.stdout.as_bytes()).unwrap();
                }
            } else if !output.stdout.is_empty() {
                println!("{}", output.stdout);
            } else if !output.stderr.is_empty() {
                println!("{}", output.stderr);
            }
        }
    }
}

fn handle_echo(args: impl Iterator<Item = String>) -> Output {
    Output::from_stdout(args.collect::<Vec<String>>().join(" "))
}

fn handle_type(args: impl Iterator<Item = String>) -> Output {
    args.map(|arg| find_type(arg))
        .fold(Output::new(), |acc, a| acc + a)
}

fn find_type(arg: String) -> Output {
    match Builtin::from(arg.as_str()) {
        Builtin::Command { command } => match find_executable(&command) {
            Some(target_path) => {
                Output::from_stdout(format!("{} is {}", arg, target_path.to_str().unwrap()))
            }
            None => Output::from_stderr(format!("{arg}: not found")),
        },
        _ => Output::from_stdout(format!("{} is a shell builtin", arg)),
    }
}

fn handle_cd(args: impl Iterator<Item = String>) -> Output {
    let mut args = args;
    let arg = args.next().unwrap_or("~".to_string());
    if let Some(_) = args.next() {
        Output::from_stderr(format!("cd: too many arguments"));
    }
    let home = env::var("HOME").unwrap();
    let home_path = Path::new(&home);
    if arg == "~" {
        env::set_current_dir(home_path).unwrap();
    } else if arg.starts_with("~/") {
        match env::set_current_dir(home_path.join(&arg[2..])) {
            Ok(_) => {}
            Err(_) => return Output::from_stderr(format!("{}: No such file or directory", arg)),
        }
    } else {
        match env::set_current_dir(&arg) {
            Ok(_) => {}
            Err(_) => return Output::from_stderr(format!("{}: No such file or directory", arg)),
        }
    }
    Output::from_stdout(String::new())
}

fn find_executable(command: &str) -> Option<PathBuf> {
    for path in env::split_paths(&env::var("PATH").unwrap_or_default()) {
        if let Ok(dir) = path.read_dir() {
            for entry in dir {
                if let Ok(valid_entry) = entry {
                    let valid_path = valid_entry.path();
                    if valid_path.file_name().and_then(|s| s.to_str()) == Some(command)
                        && valid_path.is_executable()
                    {
                        return Some(valid_path);
                    }
                }
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

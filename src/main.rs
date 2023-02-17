// rush is a rusty toy shell

use std::io;
use std::process::Command;
use std::{env, io::Write};

fn current_dir() -> String {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.to_str().unwrap();
    current_dir.to_string()
}

fn make_env_vars_map() -> std::collections::HashMap<String, String> {
    let mut env_vars_map = std::collections::HashMap::new();
    for (key, value) in std::env::vars() {
        env_vars_map.insert(key, value);
    }
    env_vars_map
}

fn give_prompt() {
    print!("{:?} > ", current_dir());
    std::io::stdout().flush().expect("Failed to flush stdout");
}

fn get_user_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input
}

struct Quoting {
    active: bool,
    quote_type: Option<char>,
}

impl Quoting {
    fn new() -> Quoting {
        Quoting {
            active: false,
            quote_type: None,
        }
    }
}

fn is_termination_char(c: char) -> bool {
    c == ' ' || c == '\n'
}

fn is_quoting_char(c: char) -> bool {
    c == '\'' || c == '"'
}

fn parse_command(input: &str) -> Vec<String> {
    let mut quoting = Quoting::new();
    let mut command = String::new();
    let mut commands = Vec::new();
    for c in input.chars() {
        if is_termination_char(c) && !quoting.active {
            commands.push(command);
            command = String::new();
        } else if is_quoting_char(c) {
            if quoting.active {
                if quoting.quote_type == Some(c) {
                    quoting.active = false;
                    quoting.quote_type = None;
                } else {
                    command.push(c);
                }
            } else {
                quoting.active = true;
                quoting.quote_type = Some(c);
            }
        } else {
            command.push(c);
        }
    }
    commands
}

fn main() {
    let env_var_map = make_env_vars_map();
    print!("env_var_map:\n {:?}", env_var_map);
    loop {
        give_prompt();
        let input = get_user_input();
        let commands = parse_command(&input);
        println!("commands: {:?}", commands);
        let mut user_cmd = Command::new(&commands[0]);
        for command in commands.into_iter() {
            user_cmd.arg(&command);
        }

        let out = user_cmd.output();
        match out {
            Ok(out) => {
                io::stdout().write_all(&out.stdout).unwrap();
            }
            Err(e) => {
                let mut err = e.to_string();
                err.push_str("\n");
                io::stderr().write_all((err).as_bytes()).unwrap();
            }
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let input = "ls -l\n";
        let commands = parse_command(input);
        println!("test commands: {:?}", commands);
        assert_eq!(commands, vec!["ls", "-l"]);
    }
}
// rush is a rusty toy shell

use std::io;
use std::process::Command;
use std::{env, io::Write};

// My preferred prompt needs this fn
#[allow(dead_code)]
fn current_dir() -> String {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.to_str().unwrap();
    current_dir.to_string()
}

fn give_prompt() {
    // Leaving this here to easily switch to preferred prompt
    // let prompt = format!("{} > ", current_dir());
    let prompt = format!("$ ");
    // output prompt to stderr
    io::stderr().write_all((prompt).as_bytes()).unwrap();
}

fn get_user_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
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

fn parse_command(input: &str) -> Result<Vec<String>, &str> {
    let mut quoting = Quoting::new();
    let mut command = String::new();
    let mut commands = Vec::new();
    let mut length = 0;
    for c in input.chars() {
        length += 1;
        if length > 1000 {
            return Err("error: command too long");
        }
        if is_termination_char(c) && !quoting.active {
            let candidate = command.trim().to_string();
            if candidate.len() > 0 {
                commands.push(candidate);
            }
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
    if command.len() > 0 {
        commands.push(command.trim().to_string());
    }
    Ok(commands)
}

fn handle_exit_code(code: i32) {
    if code != 0 {
        let err_msg = format!("error: command exited with error code {}\n", code);
        io::stderr().write_all(err_msg.as_bytes()).unwrap();
    }
}

fn handle_output(out: std::io::Result<std::process::Output>) {
    match out {
        Ok(out) => {
            io::stdout().write_all(&out.stdout).unwrap();
            io::stderr().write_all(&out.stderr).unwrap();
            handle_exit_code(out.status.code().unwrap())
        }
        Err(e) => {
            let mut err = e.to_string();
            err.push_str("\n");
            io::stderr().write_all((err).as_bytes()).unwrap();
            handle_exit_code(e.raw_os_error().unwrap());
        }
    }
}

fn main() {
    loop {
        give_prompt();
        let input = get_user_input();

        // Match input to check for "exit" or empty line
        match input.as_str() {
            "exit\n" => break,
            "\n" => continue,
            "" => continue,
            _ => (),
        }

        // Tokenize input and convert to &[&str]
        let tokens = match parse_command(&input) {
            Ok(tokens) => tokens,
            Err(_) => {
                continue;
            }
        };
        let commands: &[&str] = &tokens.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

        // Match commands to check for "cd"
        match commands {
            ["cd", dir] => {
                let dir = env::current_dir().unwrap().join(dir);
                env::set_current_dir(dir).expect("error: cd failed");
                continue;
            }
            ["cd", ..] => {
                io::stderr().write_all(b"error: cd failed\n").unwrap();
                continue;
            }
            _ => (),
        }

        // Try to run command and return output
        let mut user_cmd = Command::new(commands[0]);
        for command in commands[1..].into_iter() {
            user_cmd.arg(command);
        }
        handle_output(user_cmd.output());
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let input = "ls -l\n";
        let commands = parse_command(input).unwrap();
        assert_eq!(commands, vec!["ls", "-l"]);
    }

    #[test]
    fn test_parse_command_with_quotes() {
        let input = "ls -l 'foo bar'\n";
        let commands = parse_command(input).unwrap();
        assert_eq!(commands, vec!["ls", "-l", "foo bar"]);
    }

    #[test]
    fn test_parse_command_with_quotes_and_spaces() {
        let input = "ls -l 'foo bar'        baz\n";
        let commands = parse_command(input).unwrap();
        assert_eq!(commands, vec!["ls", "-l", "foo bar", "baz"]);
    }

    #[test]
    fn test_parse_command_with_leading_spaces_and_tabs() {
        let input = "	 ls -l 'foo bar'";
        let commands = parse_command(input).unwrap();
        assert_eq!(commands, vec!["ls", "-l", "foo bar"]);
    }
}

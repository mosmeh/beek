use crate::interpreter::env::{Environment, NamedItem};
use crate::interpreter::exec_stmt;
use crate::language::{parse, Identifier};
use colored::Colorize;
use itertools::Itertools;
use std::fmt::{self, Display};

static COMMANDS: &[&str] = &[
    "help", "?", "list", "ls", "ll", "dir", "delete", "del", "rm", "reset", "clear", "cls", "quit",
    "exit",
];

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Help,
    List,
    Delete(Identifier),
    Reset,
    Clear,
    Quit,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Self::Help => write!(f, "help"),
            Self::List => write!(f, "list"),
            Self::Delete(var) => write!(f, "delete {}", var),
            Self::Reset => write!(f, "reset"),
            Self::Clear => write!(f, "clear"),
            Self::Quit => write!(f, "quit"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Empty,
    Message(String),
    ClearScreen,
    Quit,
}

#[derive(Debug, Default)]
pub struct Repl {
    env: Environment,
}

impl Repl {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self, input: &str) -> Response {
        if let Some(cmd) = parse_command(input) {
            return self.exec_command(cmd);
        }

        let stmts = match parse(&input[..]) {
            Ok(x) => x,
            Err(e) => {
                return Response::Message(e.to_string().trim().red().to_string());
            }
        };

        let mut msg_lines = Vec::new();
        for stmt in stmts {
            msg_lines.push(format!("{}", stmt));

            match exec_stmt(&stmt, &mut self.env) {
                Ok(Some(value)) => {
                    msg_lines.push(format!(" = {}", value));
                }
                Err(e) => {
                    msg_lines.push(e.to_string().red().to_string());
                    return Response::Message(msg_lines.join("\n"));
                }
                _ => (),
            }
        }

        Response::Message(msg_lines.join("\n"))
    }

    pub fn completion_candidates(&self) -> impl Iterator<Item = &str> {
        COMMANDS
            .iter()
            .copied()
            .chain(self.env.iter().map(|(name, _)| name.0.as_str()))
            .sorted()
    }

    fn exec_command(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::Help => Response::Message(
                "Documentation: https://github.com/mosmeh/beek#reference".to_string(),
            ),
            Command::List => {
                let mut msg_lines = vec!["Variables and constants:".to_string()];
                msg_lines.extend(
                    self.env
                        .iter()
                        .filter_map(|(name, item)| {
                            if let NamedItem::Field(field) = item {
                                Some((name, field))
                            } else {
                                None
                            }
                        })
                        .sorted_by(|(a_var, a_value), (b_var, b_value)| {
                            a_value
                                .partial_cmp(&b_value)
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| a_var.cmp(&b_var))
                        })
                        .group_by(|(_, value)| *value)
                        .into_iter()
                        .map(|(value, vars)| {
                            let vars = vars.map(|(var, _)| var).join(" = ");
                            format!("{} = {}", vars, value)
                        })
                        .sorted(),
                );

                Response::Message(msg_lines.join("\n"))
            }
            Command::Delete(var) => match self.env.delete(&var) {
                Ok(_) => Response::Empty,
                Err(e) => Response::Message(e.to_string().red().to_string()),
            },
            Command::Reset => {
                self.env = Environment::new();
                Response::Empty
            }
            Command::Clear => Response::ClearScreen,
            Command::Quit => Response::Quit,
        }
    }
}

fn parse_command(input: &str) -> Option<Command> {
    // TODO: support multi line input

    let mut token = input.trim().split('\n').next()?.trim().split_whitespace();
    let cmd = token.next()?.to_ascii_lowercase();
    let arg = token.next().unwrap_or("");

    match &cmd[..] {
        "help" | "?" => Some(Command::Help),
        "list" | "ls" | "ll" | "dir" => Some(Command::List),
        "delete" | "del" | "rm" => {
            let var = Identifier(arg.to_string());
            Some(Command::Delete(var))
        }
        "reset" => Some(Command::Reset),
        "clear" | "cls" => Some(Command::Clear),
        "quit" | "exit" => Some(Command::Quit),
        _ => None,
    }
}

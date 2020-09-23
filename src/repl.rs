use crate::interpreter::env::{Environment, Field, Function, NamedItem};
use crate::interpreter::exec_stmt;
use crate::language::{parse, Identifier, Number};
use colored::Colorize;
use itertools::Itertools;

static COMMANDS: &[&str] = &[
    "help", "?", "list", "ls", "ll", "dir", "delete", "del", "rm", "reset", "clear", "cls", "quit",
    "exit",
];

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Help,
    List,
    Delete(Vec<Identifier>),
    Reset,
    Clear,
    Quit,
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

    pub fn with_env(env: Environment) -> Self {
        Self { env }
    }

    pub fn run(&mut self, input: &str) -> Response {
        if let Some(cmd) = parse_command(input) {
            return self.exec_command(cmd);
        }

        let stmts = match parse(input) {
            Ok(x) => x,
            Err(e) => {
                return Response::Message(e.to_string().trim().red().to_string());
            }
        };

        if stmts.is_empty() {
            return Response::Empty;
        }

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
                let msg_consts =
                    format_fields(self.env.iter().filter_map(|(name, item)| match item {
                        NamedItem::Field(Field::Constant(value)) => Some((name, value)),
                        _ => None,
                    }));

                let msg_vars =
                    format_fields(self.env.iter().filter_map(|(name, item)| match item {
                        NamedItem::Field(Field::Variable(value)) => Some((name, value)),
                        _ => None,
                    }));

                let msg_funcs = self
                    .env
                    .iter()
                    .filter_map(|(name, item)| match item {
                        NamedItem::Function(Function::UserDefined { arg_names, .. }) => {
                            let args = arg_names
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            Some(format!("{}({})", name, args))
                        }
                        _ => None,
                    })
                    .sorted()
                    .collect::<Vec<_>>()
                    .join(", ");

                Response::Message(format!(
                    r#"Constants:
{}

Variables:
{}

User-defined functions:
{}"#,
                    msg_consts, msg_vars, msg_funcs
                ))
            }
            Command::Delete(idents) => {
                let errors: Vec<_> = idents
                    .into_iter()
                    .filter_map(|ident| self.env.delete(&ident).err())
                    .map(|err| err.to_string())
                    .collect();
                if errors.is_empty() {
                    Response::Empty
                } else {
                    Response::Message(errors.join("\n").red().to_string())
                }
            }
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

    let mut tokens = input.trim().split('\n').next()?.trim().split_whitespace();
    let cmd = tokens.next()?.to_ascii_lowercase();

    match &cmd[..] {
        "help" | "?" => Some(Command::Help),
        "list" | "ls" | "ll" | "dir" => Some(Command::List),
        "delete" | "del" | "rm" => {
            let idents: Vec<_> = tokens.map(|ident| Identifier(ident.to_string())).collect();
            Some(Command::Delete(idents))
        }
        "reset" => Some(Command::Reset),
        "clear" | "cls" => Some(Command::Clear),
        "quit" | "exit" => Some(Command::Quit),
        _ => None,
    }
}

fn format_fields<'a>(iter: impl Iterator<Item = (&'a Identifier, &'a Number)>) -> String {
    iter.sorted_by(|(a_name, a_value), (b_name, b_value)| {
        a_value
            .partial_cmp(&b_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a_name.cmp(&b_name))
    })
    .group_by(|(_, value)| *value)
    .into_iter()
    .map(|(value, fields)| {
        let names = fields.map(|(name, _)| name).join(" = ");
        format!("{} = {}", names, value)
    })
    .sorted()
    .collect::<Vec<_>>()
    .join("\n")
}

use crate::interpreter::*;
use crate::language::*;
use colored::*;
use itertools::Itertools;
use std::fmt::{self, Display};

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

    pub fn run(&mut self, line: &str) -> Response {
        if let Some(cmd) = parse_command(line) {
            return self.exec_command(cmd);
        }

        let stmts = match parse(&line[..]) {
            Ok(x) => x,
            Err(e) => {
                return Response::Message(e.to_string().trim().red().to_string());
            }
        };

        let mut lines = Vec::new();
        for stmt in stmts {
            lines.push(format!("{}", stmt));

            match exec_stmt(&stmt, &mut self.env) {
                Ok(value) => {
                    lines.push(self.format_eval_steps(&stmt, &value));
                }
                Err(e) => {
                    lines.push(e.to_string().red().to_string());
                    return Response::Message(lines.join("\n"));
                }
            }
        }

        Response::Message(lines.join("\n"))
    }

    fn exec_command(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::Help => Response::Message("help list delete reset clear quit".to_string()),
            Command::List => {
                let mut lines = self
                    .env
                    .iter_idents()
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
                    .sorted();

                Response::Message(lines.join("\n"))
            }
            Command::Delete(var) => match self.env.delete_ident(&var) {
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

    fn format_eval_steps(&self, stmt: &Statement, final_value: &Expression) -> String {
        let expr = match stmt {
            Statement::Expression(expr) => expr,
            Statement::LazyAssignment(Assignment { var: _, expr }) => expr,
            Statement::ImmediateAssignment(Assignment { var: _, expr }) => expr,
        };
        let mut expr = expr.clone();

        let mut steps = Vec::new();
        loop {
            let before = expr.clone();

            for f in &[expand_expr_once, eval_expr_once] {
                if let Ok(evaluated) = f(&expr, &self.env) {
                    if evaluated != expr {
                        steps.push(evaluated.clone());
                        if steps.len() >= 3 {
                            break;
                        }

                        expr = evaluated;
                    }
                } else {
                    return format!(" = {}", final_value);
                }
            }

            if expr == before {
                break;
            }
        }

        match steps.len() {
            0 | 1 => format!(" = {}", final_value),
            2 => format!(" = {} = {}", steps[0], final_value),
            _ => format!(" = {} = ... = {}", steps[0], final_value),
        }
    }
}

fn parse_command(line: &str) -> Option<Command> {
    let mut token = line.trim().split_whitespace();
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

fn expand_expr_once(expr: &Expression, env: &Environment) -> EvalResult<Expression> {
    Ok(match expr {
        Expression::Identifier(ident) => env.resolve_ident(ident).unwrap_or(expr).clone(),
        Expression::UnaryOp(op, x) => {
            let x = expand_expr_once(x, env)?;
            Expression::UnaryOp(*op, Box::new(x))
        }
        Expression::BinaryOp(op, a, b) => {
            let (a, b) = (expand_expr_once(a, env)?, expand_expr_once(b, env)?);
            Expression::BinaryOp(*op, Box::new(a), Box::new(b))
        }
        _ => expr.clone(),
    })
}

fn eval_expr_once(expr: &Expression, env: &Environment) -> EvalResult<Expression> {
    match expr {
        Expression::UnaryOp(op, x) => {
            let x = eval_expr_once(x, env)?;
            if let Expression::Number(x) = x {
                Ok(Expression::Number(op.apply(x)?))
            } else {
                Ok(Expression::UnaryOp(*op, Box::new(x)))
            }
        }
        Expression::BinaryOp(op, a, b) => {
            let (a, b) = (eval_expr_once(a, env)?, eval_expr_once(b, env)?);
            if let Expression::Number(a) = a {
                if let Expression::Number(b) = b {
                    return Ok(Expression::Number(op.apply(a, b)?));
                }
            }
            Ok(Expression::BinaryOp(*op, Box::new(a), Box::new(b)))
        }
        _ => Ok(expr.clone()),
    }
}

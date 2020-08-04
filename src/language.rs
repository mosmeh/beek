mod parser;
pub use parser::parse;

use colored::*;
use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Number(pub f64);

impl From<f64> for Number {
    fn from(x: f64) -> Self {
        Self(x)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.to_string().cyan())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.to_string().yellow())
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(Expression),
    LazyAssignment(Assignment),
    ImmediateAssignment(Assignment),
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Expression(expr) => write!(f, "{}", expr),
            Self::LazyAssignment(Assignment { var, expr }) => {
                write!(f, "{} = {}", var.to_string().yellow(), expr)
            }
            Self::ImmediateAssignment(Assignment { var, expr }) => {
                write!(f, "{} := {}", var.to_string().yellow(), expr)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Negate,
    Factorial,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "×",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::Power => "^",
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Number(Number),
    Identifier(Identifier),
    UnaryOp(UnaryOp, Box<Expression>),
    BinaryOp(BinaryOp, Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Number(x) => write!(f, "{}", x),
            Self::Identifier(x) => write!(f, "{}", x),
            Self::UnaryOp(op, x) => {
                if *op == UnaryOp::Negate {
                    write!(f, "-")?;
                }

                match **x {
                    Self::UnaryOp(_, _) | Self::BinaryOp(_, _, _) => {
                        write!(f, "({})", x)?;
                    }
                    _ => {
                        write!(f, "{}", x)?;
                    }
                };

                if *op == UnaryOp::Factorial {
                    write!(f, "!")?;
                }

                Ok(())
            }
            Self::BinaryOp(op, a, b) => {
                if let Self::BinaryOp(_, _, _) = **a {
                    write!(f, "({})", a)?
                } else {
                    write!(f, "{}", a)?
                }

                write!(f, " {} ", op)?;

                if let Self::BinaryOp(_, _, _) = **b {
                    write!(f, "({})", b)
                } else {
                    write!(f, "{}", b)
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub var: Identifier,
    pub expr: Expression,
}

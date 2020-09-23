mod parser;
pub use parser::parse;

use colored::Colorize;
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
        let mut buffer = ryu::Buffer::new();
        let formatted = buffer.format(self.0);
        let formatted = formatted.strip_suffix(".0").unwrap_or(formatted);
        write!(f, "{}", formatted.cyan())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.to_string().yellow())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    VariableAssignment(VariableAssignment),
    FunctionDefinition(FunctionDefinition),
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Expression(expr) => write!(f, "{}", expr),
            Self::VariableAssignment(assign) => write!(f, "{}", assign),
            Self::FunctionDefinition(def) => write!(f, "{}", def),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Negate,
    Factorial,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            Self::Negate => "-",
            Self::Factorial => "!",
        })
    }
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

impl BinaryOp {
    fn precedence(self) -> u8 {
        match self {
            BinaryOp::Add | BinaryOp::Subtract => 0,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 1,
            BinaryOp::Power => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Number(Number),
    Variable(Identifier),
    Function(Identifier, Vec<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
    BinaryOp(BinaryOp, Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Number(x) => write!(f, "{}", x),
            Self::Variable(x) => write!(f, "{}", x),
            Self::Function(name, xs) => write!(
                f,
                "{}({})",
                name,
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::UnaryOp(op, x) => {
                if *op == UnaryOp::Negate {
                    write!(f, "{}", UnaryOp::Negate)?;
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
                    write!(f, "{}", UnaryOp::Factorial)?;
                }

                Ok(())
            }
            Self::BinaryOp(BinaryOp::Power, a, b) => {
                // show parentheses regardless of precedences to clarify right-associativity
                match **a {
                    Self::UnaryOp(_, _) | Self::BinaryOp(_, _, _) => write!(f, "({})", a)?,
                    _ => write!(f, "{}", a)?,
                };

                write!(f, "{}", BinaryOp::Power)?;

                match **b {
                    Self::UnaryOp(_, _) | Self::BinaryOp(_, _, _) => write!(f, "({})", b),
                    _ => write!(f, "{}", b),
                }
            }
            Self::BinaryOp(op, a, b) => {
                match **a {
                    Self::BinaryOp(sub_op, _, _) if sub_op.precedence() < op.precedence() => {
                        write!(f, "({})", a)?
                    }
                    _ => write!(f, "{}", a)?,
                };

                write!(f, " {} ", op)?;

                match **b {
                    Self::BinaryOp(sub_op, _, _) if sub_op.precedence() <= op.precedence() => {
                        write!(f, "({})", b)
                    }
                    _ => write!(f, "{}", b),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableAssignment {
    pub name: Identifier,
    pub expr: Expression,
}

impl Display for VariableAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} = {}", self.name, self.expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub arg_names: Vec<Identifier>,
    pub expr: Expression,
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}({}) = {}",
            self.name,
            self.arg_names
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.expr
        )
    }
}

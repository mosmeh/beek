use super::language::*;
use std::collections::HashMap;
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Encountered infinity or NaN")]
    NumericalError,
    #[error("{0}")]
    AssignError(String),
    #[error("Unknown identifier: {0}")]
    ReferenceError(Identifier),
}

pub type EvalResult<T> = std::result::Result<T, EvalError>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NamedValue {
    Variable(Expression),
    Constant(Expression),
}

impl Display for NamedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Variable(x) => write!(f, "{}", x),
            Self::Constant(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Environment(HashMap<Identifier, NamedValue>);

impl Default for Environment {
    fn default() -> Self {
        use std::f64::consts::*;
        Self(
            vec![("ans", 0.0), ("_", 0.0), ("pi", PI), ("π", PI), ("e", E)]
                .into_iter()
                .map(|(var, x)| {
                    let expr = Expression::Number(Number(x));
                    (Identifier(var.to_string()), NamedValue::Constant(expr))
                })
                .collect(),
        )
    }
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn iter_idents(&self) -> impl Iterator<Item = (&Identifier, &NamedValue)> {
        self.0.iter()
    }

    pub fn delete_ident(&mut self, ident: &Identifier) -> Result<(), EvalError> {
        if self.0.remove(ident).is_some() {
            Ok(())
        } else {
            Err(EvalError::ReferenceError(ident.clone()))
        }
    }

    pub fn resolve_ident(&self, ident: &Identifier) -> Option<&Expression> {
        self.0.get(ident).map(|value| match value {
            NamedValue::Variable(x) => x,
            NamedValue::Constant(x) => x,
        })
    }

    pub fn assign_var(&mut self, ident: &Identifier, expr: &Expression) -> Result<(), EvalError> {
        match self.0.get_mut(ident) {
            Some(NamedValue::Variable(var)) => {
                *var = expr.clone();
                Ok(())
            }
            Some(NamedValue::Constant(_)) => Err(EvalError::AssignError(format!(
                "Cannot assign to a constant {}",
                ident
            ))),
            None => {
                self.0
                    .insert(ident.clone(), NamedValue::Variable(expr.clone()));
                Ok(())
            }
        }
    }
}

pub fn exec_stmt(stmt: &Statement, env: &mut Environment) -> EvalResult<Expression> {
    let value = match stmt {
        Statement::Expression(expr) => eval_expr(expr, env)?,
        Statement::LazyAssignment(assignment) => lazy_assign(assignment, env)?,
        Statement::ImmediateAssignment(assignment) => immediate_assign(assignment, env)?,
    };
    for var in &["ans", "_"] {
        env.0.insert(
            Identifier(var.to_string()),
            NamedValue::Constant(value.clone()),
        );
    }

    Ok(value)
}

fn eval_expr(expr: &Expression, env: &Environment) -> EvalResult<Expression> {
    match expr {
        Expression::Number(_) => Ok(expr.clone()),
        Expression::Identifier(ident) => {
            if let Some(x) = env.resolve_ident(ident) {
                eval_expr(x, env)
            } else {
                Ok(expr.clone())
            }
        }
        Expression::UnaryOp(op, x) => {
            let x = eval_expr(x, env)?;
            if let Expression::Number(x) = x {
                Ok(Expression::Number(op.apply(x)?))
            } else {
                Ok(Expression::UnaryOp(*op, Box::new(x)))
            }
        }
        Expression::BinaryOp(op, a, b) => {
            let (a, b) = (eval_expr(a, env)?, eval_expr(b, env)?);
            if let Expression::Number(a) = a {
                if let Expression::Number(b) = b {
                    return Ok(Expression::Number(op.apply(a, b)?));
                }
            }
            Ok(Expression::BinaryOp(*op, Box::new(a), Box::new(b)))
        }
    }
}

fn lazy_assign(assignment: &Assignment, env: &mut Environment) -> EvalResult<Expression> {
    let Assignment { var, expr } = assignment;

    // TODO: detect recursive definition and eval simultaneously
    if expr_contains_ident(expr, var, env) {
        return Err(EvalError::AssignError(format!(
            "Detected recursive definition of variable {}",
            var
        )));
    }

    let evaluated = eval_expr(expr, env)?;
    env.assign_var(var, expr)?;
    Ok(evaluated)
}

fn immediate_assign(assignment: &Assignment, env: &mut Environment) -> EvalResult<Expression> {
    let Assignment { var, expr } = assignment;
    if expr_contains_ident(expr, var, env) {
        return Err(EvalError::AssignError(format!(
            "Detected recursive definition of variable {}",
            var
        )));
    }

    let evaluated = eval_expr(expr, env)?;
    env.assign_var(var, expr)?;
    Ok(evaluated)
}

impl UnaryOp {
    pub fn apply(self, x: Number) -> EvalResult<Number> {
        let value = match self {
            Self::Negate => -x.0,
            Self::Factorial => factorial(x.0),
        };

        if value.is_finite() {
            Ok(Number(value))
        } else {
            Err(EvalError::NumericalError)
        }
    }
}

impl BinaryOp {
    pub fn apply(self, a: Number, b: Number) -> EvalResult<Number> {
        let (a, b) = (a.0, b.0);
        let value = match self {
            Self::Add => a + b,
            Self::Subtract => a - b,
            Self::Multiply => a * b,
            Self::Divide => a / b,
            Self::Modulo => a % b,
            Self::Power => a.powf(b),
        };

        if value.is_finite() {
            Ok(Number(value))
        } else {
            Err(EvalError::NumericalError)
        }
    }
}

fn factorial(x: f64) -> f64 {
    use statrs::function::*;

    if x >= 0.0 && x.fract() == 0.0 {
        factorial::factorial(x as u64)
    } else {
        gamma::gamma(x + 1.0)
    }
}

fn expr_contains_ident(expr: &Expression, ident: &Identifier, env: &Environment) -> bool {
    match expr {
        Expression::Identifier(x) if x == ident => true,
        Expression::Identifier(x) => {
            if let Some(x) = env.resolve_ident(x) {
                expr_contains_ident(x, ident, env)
            } else {
                false
            }
        }
        Expression::UnaryOp(_, x) => expr_contains_ident(x, ident, env),
        Expression::BinaryOp(_, a, b) => {
            expr_contains_ident(a, ident, env) || expr_contains_ident(b, ident, env)
        }
        _ => false,
    }
}

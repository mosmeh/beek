pub mod env;

use crate::language::{
    BinaryOp, Expression, FunctionDefinition, Identifier, Number, Statement, UnaryOp,
    VariableAssignment,
};
use env::{Environment, Function};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Encountered non-finite value: {0}")]
    NumericalError(Number),

    #[error("{0}")]
    TypeError(String),

    #[error("Unknown identifier {0}")]
    ReferenceError(Identifier),

    #[error("The function {name} takes {expected} {} but {got} {} supplied",
            if *.expected == 1 { "argument" } else { "arguments" },
            if *.got == 1 { "was" } else { "were" }
        )]
    ArityError {
        name: String,
        expected: usize,
        got: usize,
    },

    #[error("{0}")]
    DefinitionError(String),
}

pub type EvalResult<T> = Result<T, EvalError>;

pub fn exec_stmt(stmt: &Statement, env: &mut Environment) -> EvalResult<Option<Number>> {
    let value = match stmt {
        Statement::Expression(expr) => Some(eval_expr(expr, env)?),
        Statement::VariableAssignment(VariableAssignment { name, expr }) => {
            let evaluated = eval_expr(expr, env)?;
            env.assign_var(name, evaluated)?;
            Some(evaluated)
        }
        Statement::FunctionDefinition(FunctionDefinition {
            name,
            arg_names,
            expr,
        }) => {
            env.def_func(name, &arg_names, expr)?;
            None
        }
    };

    if let Some(value) = value {
        for name in &["ans", "_"] {
            env.def_const(&Identifier(name.to_string()), value)?;
        }
    }

    Ok(value)
}

fn eval_expr(expr: &Expression, env: &Environment) -> EvalResult<Number> {
    let value = match expr {
        Expression::Number(x) => *x,
        Expression::Field(name) => env.resolve_field(name)?,
        Expression::Function(name, xs) => {
            let func = env.resolve_func(name)?;
            let args = xs
                .iter()
                .map(|x| eval_expr(x, env))
                .collect::<EvalResult<Vec<Number>>>()?;
            eval_func(name, func, &args, env)?
        }
        Expression::UnaryOp(op, x) => {
            let x = eval_expr(x, env)?;
            op.apply(x)?
        }
        Expression::BinaryOp(op, a, b) => {
            let (a, b) = (eval_expr(a, env)?, eval_expr(b, env)?);
            op.apply(a, b)?
        }
    };

    if value.0.is_finite() {
        Ok(value)
    } else {
        Err(EvalError::NumericalError(value))
    }
}

fn eval_func(
    name: &Identifier,
    func: &Function,
    args: &[Number],
    env: &Environment,
) -> EvalResult<Number> {
    if args.len() != func.num_args() {
        return Err(EvalError::ArityError {
            name: name.to_string(),
            expected: func.num_args(),
            got: args.len(),
        });
    }

    match func {
        Function::NullaryBuiltin(ptr) => Ok(Number(ptr())),
        Function::UnaryBuiltin(ptr) => Ok(Number(ptr(args[0].0))),
        Function::BinaryBuiltin(ptr) => Ok(Number(ptr(args[0].0, args[1].0))),
        Function::UserDefined { arg_names, expr } => {
            let mut env = env.clone();
            env.delete(name).unwrap(); // avoid infinite recursion
            for (name, value) in arg_names.iter().zip(args.iter()) {
                env.def_const(name, *value)?;
            }

            eval_expr(&expr, &env)
        }
    }
}

impl UnaryOp {
    pub fn apply(self, x: Number) -> EvalResult<Number> {
        let value = match self {
            Self::Negate => -x.0,
            Self::Factorial => factorial(x.0),
        };
        Ok(Number(value))
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
        Ok(Number(value))
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

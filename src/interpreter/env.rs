use super::{EvalError, EvalResult};
use crate::language::{Expression, Identifier, Number};
use rand::Rng;
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Clone)]
pub enum Field {
    Variable(Number),
    Constant(Number),
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(x) => write!(f, "{}", x),
            Self::Constant(x) => write!(f, "{}", x),
        }
    }
}

impl Field {
    fn inner(self) -> Number {
        match self {
            Self::Variable(x) => x,
            Self::Constant(x) => x,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Function {
    NullaryBuiltin(fn() -> f64),
    UnaryBuiltin(fn(f64) -> f64),
    BinaryBuiltin(fn(f64, f64) -> f64),
    UserDefined {
        arg_names: Vec<Identifier>,
        expr: Expression,
    },
}

impl Function {
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::UserDefined { .. })
    }

    pub fn num_args(&self) -> usize {
        match self {
            Self::NullaryBuiltin(_) => 0,
            Self::UnaryBuiltin(_) => 1,
            Self::BinaryBuiltin(_) => 2,
            Self::UserDefined { arg_names, .. } => arg_names.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NamedItem {
    Field(Field),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct Environment(HashMap<Identifier, NamedItem>);

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn resolve_field(&self, ident: &Identifier) -> EvalResult<Number> {
        match self.0.get(ident) {
            Some(NamedItem::Field(field)) => Ok(field.clone().inner()),
            Some(NamedItem::Function(_)) => Err(EvalError::TypeError(format!(
                "{} is not a variable or constant",
                ident
            ))),
            None => Err(EvalError::ReferenceError(ident.clone())),
        }
    }

    pub fn resolve_func(&self, ident: &Identifier) -> EvalResult<&Function> {
        match self.0.get(ident) {
            Some(NamedItem::Function(func)) => Ok(func),
            Some(NamedItem::Field(_)) => {
                Err(EvalError::TypeError(format!("{} is not a function", ident)))
            }
            None => Err(EvalError::ReferenceError(ident.clone())),
        }
    }

    pub fn delete(&mut self, ident: &Identifier) -> EvalResult<()> {
        match self.0.get(ident) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot delete a constant {}",
                ident
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot delete a built-in function {}", ident),
            )),
            None => Err(EvalError::ReferenceError(ident.clone())),
            _ => {
                self.0.remove(ident).unwrap();
                Ok(())
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &NamedItem)> {
        self.0.iter()
    }

    pub fn assign_var(&mut self, name: &Identifier, value: Number) -> EvalResult<()> {
        match self.0.get(name) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot assign to a constant {}",
                name
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot redefine a built-in function {}", name),
            )),
            _ => {
                self.0
                    .insert(name.clone(), NamedItem::Field(Field::Variable(value)));
                Ok(())
            }
        }
    }

    pub fn def_const(&mut self, name: &Identifier, value: Number) -> EvalResult<()> {
        self.0
            .insert(name.clone(), NamedItem::Field(Field::Constant(value)));
        Ok(())
    }

    pub fn def_func(
        &mut self,
        name: &Identifier,
        arg_names: &[Identifier],
        expr: &Expression,
    ) -> EvalResult<()> {
        if let Some(dup) = find_duplicate(arg_names) {
            return Err(EvalError::DefinitionError(format!(
                "Duplicate argument {}",
                dup
            )));
        }

        match self.0.get(name) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot assign to a constant {}",
                name
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot redefine a built-in function {}", name),
            )),
            _ => {
                self.0.insert(
                    name.clone(),
                    NamedItem::Function(Function::UserDefined {
                        arg_names: arg_names.to_vec(),
                        expr: expr.clone(),
                    }),
                );
                Ok(())
            }
        }
    }
}

fn find_duplicate(xs: &[Identifier]) -> Option<&Identifier> {
    let mut uniq = HashSet::new();
    for x in xs.iter() {
        if !uniq.insert(x) {
            return Some(x);
        }
    }
    None
}

impl Default for Environment {
    fn default() -> Self {
        use std::f64::consts::*;

        type NullaryFunc = (&'static str, fn() -> f64);
        type UnaryFunc = (&'static str, fn(f64) -> f64);
        type BinaryFunc = (&'static str, fn(f64, f64) -> f64);

        const CONSTS: &[(&str, f64)] = &[("e", E), ("pi", PI), ("π", PI), ("tau", TAU), ("τ", TAU)];
        const NULLARY_FUNCS: &[NullaryFunc] = &[("random", random)];
        const UNARY_FUNCS: &[UnaryFunc] = &[
            ("floor", f64::floor),
            ("ceil", f64::ceil),
            ("round", f64::round),
            ("trunc", f64::trunc),
            ("fract", f64::fract),
            ("abs", f64::abs),
            ("sqrt", f64::sqrt),
            ("exp", f64::exp),
            ("log", f64::ln),
            ("ln", f64::ln),
            ("log2", f64::log2),
            ("log10", f64::log10),
            ("cbrt", f64::cbrt),
            ("sin", f64::sin),
            ("cos", f64::cos),
            ("tan", f64::tan),
            ("asin", f64::asin),
            ("acos", f64::acos),
            ("atan", f64::atan),
            ("sinh", f64::sinh),
            ("cosh", f64::cosh),
            ("tanh", f64::tanh),
            ("asinh", f64::asinh),
            ("acosh", f64::acosh),
            ("atanh", f64::atanh),
            ("degrees", f64::to_degrees),
            ("radians", f64::to_radians),
            ("erf", statrs::function::erf::erf),
            ("erfc", statrs::function::erf::erfc),
            ("gamma", statrs::function::gamma::gamma),
            ("lgamma", statrs::function::gamma::ln_gamma),
            ("sign", sign),
        ];
        const BINARY_FUNCS: &[BinaryFunc] = &[
            ("pow", f64::powf),
            ("hypot", f64::hypot),
            ("atan2", f64::atan2),
            ("max", f64::max),
            ("min", f64::min),
        ];

        let consts = CONSTS.iter().map(|(name, value)| {
            (
                Identifier(name.to_string()),
                NamedItem::Field(Field::Constant(Number(*value))),
            )
        });
        let nullary_funcs = NULLARY_FUNCS.iter().map(|(name, ptr)| {
            (
                Identifier(name.to_string()),
                NamedItem::Function(Function::NullaryBuiltin(*ptr)),
            )
        });
        let unary_funcs = UNARY_FUNCS.iter().map(|(name, ptr)| {
            (
                Identifier(name.to_string()),
                NamedItem::Function(Function::UnaryBuiltin(*ptr)),
            )
        });
        let binary_funcs = BINARY_FUNCS.iter().map(|(name, ptr)| {
            (
                Identifier(name.to_string()),
                NamedItem::Function(Function::BinaryBuiltin(*ptr)),
            )
        });
        Environment(
            consts
                .chain(nullary_funcs)
                .chain(unary_funcs)
                .chain(binary_funcs)
                .collect(),
        )
    }
}

fn random() -> f64 {
    rand::thread_rng().gen()
}

fn sign(x: f64) -> f64 {
    if x == 0.0 {
        if x.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        x.signum()
    }
}

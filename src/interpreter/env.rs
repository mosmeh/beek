use super::{EvalError, EvalResult};
use crate::language::{Expression, Identifier, Number};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Field {
    Variable(Number),
    Constant(Number),
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Function {
    NullaryBuiltin(fn() -> f64),
    UnaryBuiltin(fn(f64) -> f64),
    UserDefined {
        arg_names: Vec<Identifier>,
        expr: Expression,
    },
}

impl Function {
    fn is_builtin(&self) -> bool {
        match self {
            Self::NullaryBuiltin(_) | Self::UnaryBuiltin(_) => true,
            Self::UserDefined { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NamedItem {
    Field(Field),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct Environment(HashMap<Identifier, NamedItem>);

lazy_static::lazy_static! {
static ref DEFAULT_ENV: Environment = {
    use std::f64::consts::*;
    let values = vec![("pi", PI), ("π", PI), ("e", E)]
        .into_iter()
        .map(|(name, value)| {
            (
                Identifier(name.to_string()),
                NamedItem::Field(Field::Constant(Number(value))),
            )
        });

    type NullaryFunc = (&'static str, fn() -> f64);
    static NULLARY_FUNCS: &[NullaryFunc] = &[("rand", || rand::thread_rng().gen())];

    let nullary_funcs = NULLARY_FUNCS.iter().map(|(name, ptr)| {
        (
            Identifier(name.to_string()),
            NamedItem::Function(Function::NullaryBuiltin(*ptr)),
        )
    });

    type UnaryFunc = (&'static str, fn(f64) -> f64);
    static UNARY_FUNCS: &[UnaryFunc] = &[
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
        ("gamma", statrs::function::gamma::gamma),
    ];

    let unary_funcs = UNARY_FUNCS.iter().map(|(name, ptr)| {
        (
            Identifier(name.to_string()),
            NamedItem::Function(Function::UnaryBuiltin(*ptr)),
        )
    });

    Environment(values.chain(nullary_funcs).chain(unary_funcs).collect())
};
}

impl Default for Environment {
    fn default() -> Self {
        DEFAULT_ENV.clone()
    }
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn resolve_var(&self, ident: &Identifier) -> EvalResult<Number> {
        match self.0.get(ident) {
            Some(NamedItem::Field(field)) => Ok(field.clone().inner()),
            Some(NamedItem::Function(_)) => {
                Err(EvalError::TypeError(format!("{} is not a variable", ident)))
            }
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

    pub fn delete(&mut self, ident: &Identifier) -> Result<(), EvalError> {
        match self.0.get(ident) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot delete a constant '{}'",
                ident
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot delete a built-in function '{}'", ident),
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

    pub fn assign_var(&mut self, name: &Identifier, value: Number) -> Result<(), EvalError> {
        match self.0.get_mut(name) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot assign to a constant '{}'",
                name
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot redefine a built-in function '{}'", name),
            )),
            _ => {
                self.0
                    .insert(name.clone(), NamedItem::Field(Field::Variable(value)));
                Ok(())
            }
        }
    }

    pub fn def_const(&mut self, name: &Identifier, value: Number) -> Result<(), EvalError> {
        self.0
            .insert(name.clone(), NamedItem::Field(Field::Constant(value)));
        Ok(())
    }

    pub fn def_func<'a>(
        &mut self,
        name: &Identifier,
        arg_names: &'a [Identifier],
        expr: &Expression,
    ) -> Result<(), EvalError> {
        // TODO: avoid infinite recursion

        let find_dup = |xs: &'a [Identifier]| -> Option<&'a Identifier> {
            let mut uniq = HashSet::new();
            for x in xs.iter() {
                if !uniq.insert(x) {
                    return Some(x);
                }
            }
            None
        };

        if let Some(dup) = find_dup(&arg_names) {
            return Err(EvalError::InvalidDefinitionError(format!(
                "Duplicate argument '{}'",
                dup
            )));
        }

        match self.0.get_mut(name) {
            Some(NamedItem::Field(Field::Constant(_))) => Err(EvalError::TypeError(format!(
                "Cannot assign to a constant '{}'",
                name
            ))),
            Some(NamedItem::Function(func)) if func.is_builtin() => Err(EvalError::TypeError(
                format!("Cannot redefine a built-in function '{}'", name),
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

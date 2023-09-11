use anyhow::{anyhow, bail, Context, Result};
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{expression::ToAndFrom, token::Token, Environment, LispExpression};

pub trait Atom<E: LispExpression>: Display {
    // TODO find a better way to do this
    fn sized_name() -> &'static str
    where
        Self: Sized;

    fn name(&self) -> &'static str;

    fn call(&self, _arguments: &[E], _env: &mut Environment<E>) -> Result<E> {
        bail!("Cannot call {} as if it were a function", self.name())
    }

    fn parse_from_token(_token: &Token) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[0;32m{}\x1b[0m", self.0)
    }
}

impl<E: LispExpression> Atom<E> for Symbol {
    fn sized_name() -> &'static str {
        "symbol"
    }
    fn name(&self) -> &'static str {
        "symbol"
    }

    fn parse_from_token(token: &Token) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self(token.value.clone()))
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol(value.to_owned())
    }
}

#[derive(Clone)]
pub struct BuiltinFunction<E: 'static> {
    pub name: &'static str,
    pub function: Rc<dyn Fn(&[E], &mut Environment<E>) -> Result<E>>,
}

impl<E> BuiltinFunction<E> {
    pub fn new(name: &'static str, function: fn(&[E], &mut Environment<E>) -> Result<E>) -> Self {
        Self {
            name,
            function: Rc::new(function),
        }
    }

    // TODO What about other function signatures
    pub fn new_wrapped<U: 'static, V: 'static>(
        name: &'static str,
        function: fn(&U) -> Result<V>,
    ) -> Self
    where
        E: ToAndFrom<U> + ToAndFrom<V>,
    {
        let wrapped = move |arguments: &[E], _env: &mut Environment<E>| {
            if arguments.len() != 1 {
                bail!("Function {} must be called with a single argument", name)
            }
            let argument: &U = arguments[0]
                .try_into_atom()
                .with_context(|| anyhow!("Argument to {} is wrong type", name))?;
            function(argument).map(|v| v.into())
        };
        Self {
            name,
            function: Rc::new(wrapped),
        }
    }
}

impl<E: 'static> PartialEq for BuiltinFunction<E> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<E: LispExpression> Atom<E> for BuiltinFunction<E> {
    fn sized_name() -> &'static str {
        "builtin function"
    }
    fn name(&self) -> &'static str {
        "builtin function"
    }

    fn call(&self, arguments: &[E], env: &mut Environment<E>) -> Result<E> {
        let arguments: Vec<E> = arguments
            .iter()
            .enumerate()
            .map(|(n, e)| {
                e.eval(env)
                    .with_context(|| anyhow!("Argument number {}: {}", n + 1, e))
            })
            .collect::<Result<Vec<_>>>()
            .with_context(|| anyhow!("Could not evaluate arguments to {}", self))?;
        (self.function)(&arguments, env)
    }
}

impl<E> Debug for BuiltinFunction<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin function {}»", self.name)
    }
}

impl<E> Display for BuiltinFunction<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq)]
pub struct BuiltinMacro<E> {
    pub name: &'static str,
    pub function: fn(&[E], &mut Environment<E>) -> Result<E>,
}

impl<E> BuiltinMacro<E> {
    pub fn new(name: &'static str, function: fn(&[E], &mut Environment<E>) -> Result<E>) -> Self {
        Self { name, function }
    }
}

impl<E: LispExpression> Atom<E> for BuiltinMacro<E> {
    fn sized_name() -> &'static str {
        "builtin macro"
    }

    fn name(&self) -> &'static str {
        "builtin macro"
    }

    fn call(&self, arguments: &[E], env: &mut Environment<E>) -> Result<E> {
        (self.function)(arguments, env)
    }
}

impl<E> Debug for BuiltinMacro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin macro {}»", self.name)
    }
}

impl<E> Display for BuiltinMacro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq)]
pub struct Lambda<E> {
    pub parameters: Vec<Symbol>,
    pub value: Box<E>,
    pub env: Environment<E>,
}

impl<E: LispExpression> Atom<E> for Lambda<E> {
    fn sized_name() -> &'static str {
        "lambda"
    }

    fn name(&self) -> &'static str {
        "lambda"
    }

    fn call(&self, arguments: &[E], env: &mut Environment<E>) -> Result<E> {
        let arguments: Vec<E> = arguments
            .iter()
            .enumerate()
            .map(|(n, e)| {
                e.eval(env)
                    .with_context(|| anyhow!("Argument number {}: {}", n + 1, e))
            })
            .collect::<Result<Vec<_>>>()
            .with_context(|| anyhow!("Could not evaluate arguments to {}", self))?;
        if arguments.len() > self.parameters.len() {
            bail!("Too many arguments to lambda")
        }
        let mut env: Environment<E> = self.env.clone();
        for (parameter, argument) in self.parameters.iter().zip(&arguments) {
            env.set(parameter.clone(), argument.clone())
        }
        if arguments.len() < self.parameters.len() {
            Ok(Lambda {
                parameters: self.parameters[arguments.len()..].to_vec(),
                env,
                value: self.value.clone(),
            }
            .into())
        } else {
            self.value.eval(&mut env)
        }
    }
}

impl<E> Debug for Lambda<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lambda function")
    }
}

impl<E: Display> Display for Lambda<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parameters: Vec<String> = self.parameters.iter().map(|e| e.to_string()).collect();
        write!(f, "λ ({}) {}", parameters.join(" "), self.value)
    }
}

#[derive(Clone, PartialEq)]
pub struct Macro<E> {
    pub parameters: Vec<Symbol>,
    pub value: Box<E>,
    pub env: Environment<E>,
}

impl<E: LispExpression> Atom<E> for Macro<E> {
    fn sized_name() -> &'static str {
        "macro"
    }

    fn name(&self) -> &'static str {
        "macro"
    }

    fn call(&self, arguments: &[E], env: &mut Environment<E>) -> Result<E> {
        if arguments.len() > self.parameters.len() {
            bail!("Too many arguments to lambda")
        }
        let mut macro_env: Environment<E> = self.env.clone();
        for (parameter, argument) in self.parameters.iter().zip(arguments) {
            macro_env.set(parameter.clone(), argument.clone())
        }
        if arguments.len() < self.parameters.len() {
            Ok(Macro {
                parameters: self.parameters[arguments.len()..].to_vec(),
                env: macro_env,
                value: self.value.clone(),
            }
            .into())
        } else {
            self.value
                .eval(&mut macro_env)
                .context("Could not expand macro")?
                .eval(env)
        }
    }
}

impl<E> Debug for Macro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Macro function")
    }
}

impl<E: Display> Display for Macro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parameters: Vec<String> = self.parameters.iter().map(|e| e.to_string()).collect();
        write!(f, "μ ({}) {}", parameters.join(" "), self.value)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Number(pub f64);

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[0;36m{}\x1b[0m", self.0)
    }
}

impl<E: LispExpression> Atom<E> for Number {
    fn sized_name() -> &'static str {
        "number"
    }

    fn name(&self) -> &'static str {
        "number"
    }

    fn parse_from_token(token: &Token) -> Option<Self>
    where
        Self: Sized,
    {
        token.value.parse().ok().map(Self)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct List<E>(pub Vec<E>);

impl<E: Display> Display for List<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elements: Vec<String> = self.0.iter().map(|e| e.to_string()).collect();
        write!(f, "({})", elements.join(" "))
    }
}

impl<E: LispExpression> Atom<E> for List<E> {
    fn sized_name() -> &'static str {
        "list"
    }

    fn name(&self) -> &'static str {
        "list"
    }

    fn call(&self, arguments: &[E], _env: &mut Environment<E>) -> Result<E> {
        if arguments.len() > 1 {
            // TODO should this be the case?
            bail!("Cannot index array using more than one index")
        }
        if let Ok(number) = <E as ToAndFrom<Number>>::try_into_atom(&arguments[0]) {
            if number.0 < 0. || number.0 > self.0.len() as f64 - 1.0 {
                bail!(
                    "Cannot index array of length {} at {}",
                    self.0.len(),
                    number
                );
            }
            let index: usize = number.0 as usize;
            Ok(self.0[index].clone())
        } else {
            bail!(
                "Can only index into list using numbers, not {}",
                arguments[0]
            )
        }
    }
}

use anyhow::{anyhow, bail, Context, Result};
use std::fmt::{Debug, Display};

use crate::{Environment, Expression};

pub trait Atom {
    fn call(
        &self,
        _arguments: &[Expression],
        _env: &mut Environment<Expression>,
    ) -> Result<Expression> {
        // TODO Make this print out the self
        bail!("Cannot call ??? as if it were a function")
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

impl Atom for Symbol {}

#[derive(Clone, PartialEq)]
pub struct BuiltinFunction<E> {
    pub name: &'static str,
    pub function: fn(&[E], &mut Environment<E>) -> Result<E>,
}

impl Atom for BuiltinFunction<Expression> {
    fn call(
        &self,
        arguments: &[Expression],
        env: &mut Environment<Expression>,
    ) -> Result<Expression> {
        let arguments: Vec<Expression> = arguments
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

impl Atom for BuiltinMacro<Expression> {
    fn call(
        &self,
        arguments: &[Expression],
        env: &mut Environment<Expression>,
    ) -> Result<Expression> {
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

impl Atom for Lambda<Expression> {
    fn call(
        &self,
        arguments: &[Expression],
        env: &mut Environment<Expression>,
    ) -> Result<Expression> {
        let arguments: Vec<Expression> = arguments
            .iter()
            .enumerate()
            .map(|(n, e)| {
                e.eval(env)
                    .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
            })
            .collect::<Result<Vec<_>>>()
            .with_context(|| anyhow!("Could not evaluate arguments to {:?}", self))?;
        if arguments.len() > self.parameters.len() {
            bail!("Too many arguments to lambda")
        }
        let mut env: Environment<Expression> = self.env.clone();
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

impl Atom for Macro<Expression> {
    fn call(
        &self,
        arguments: &[Expression],
        env: &mut Environment<Expression>,
    ) -> Result<Expression> {
        if arguments.len() > self.parameters.len() {
            bail!("Too many arguments to lambda")
        }
        let mut macro_env: Environment<Expression> = self.env.clone();
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

use anyhow::Result;
use std::fmt::{Debug, Display};

use crate::Environment;

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

#[derive(Clone, PartialEq)]
pub struct BuiltinFunction<E> {
    pub name: &'static str,
    pub function: fn(&[E], &mut Environment<E>) -> Result<E>,
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

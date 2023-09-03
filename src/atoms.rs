use anyhow::Result;
use std::fmt::{Debug, Display};

use crate::Environment;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[0;32m{}\x1b[0m", self.0)
    }
}

#[derive(Clone, PartialEq)]
pub struct BuiltinFunction<E>(pub fn(&[E], &mut Environment<E>) -> Result<E>);

impl<E> Debug for BuiltinFunction<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin function»")
    }
}

impl<E> Display for BuiltinFunction<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin function»")
    }
}

#[derive(Clone, PartialEq)]
pub struct BuiltinMacro<E>(pub fn(&[E], &mut Environment<E>) -> Result<E>);

impl<E> Debug for BuiltinMacro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin macro»")
    }
}

impl<E> Display for BuiltinMacro<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "«builtin macro»")
    }
}

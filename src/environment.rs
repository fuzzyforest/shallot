use std::{collections::HashMap, fmt::Display};

use crate::atoms::Symbol;

pub struct Environment<E> {
    inner: HashMap<Symbol, E>,
}

impl<E> Default for Environment<E> {
    fn default() -> Self {
        Environment {
            inner: Default::default(),
        }
    }
}

impl<E> Environment<E> {
    pub fn get(&self, symbol: &Symbol) -> Option<&E> {
        self.inner.get(symbol)
    }

    pub fn set(&mut self, symbol: Symbol, value: impl Into<E>) {
        self.inner.insert(symbol, value.into());
    }
}

impl<E: Display> Display for Environment<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (symbol, value) in self.inner.iter() {
            writeln!(f, "{symbol} -> {value}")?;
        }
        Ok(())
    }
}

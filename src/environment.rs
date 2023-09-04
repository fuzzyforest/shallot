use std::{collections::HashMap, fmt::Display};

use crate::atoms::Symbol;

#[derive(Clone, PartialEq)]
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
        let mut all_variables = self.inner.keys().collect::<Vec<_>>();
        all_variables.sort();
        let longest_var_length = all_variables.iter().map(|s| s.len()).max().unwrap_or(0);
        let mut first = true;
        for symbol in &all_variables {
            if !first {
                writeln!(f, "")?;
            }
            first = false;
            // Note: these values exist in our map for sure
            let value = self.get(symbol).unwrap();
            let symbol = &symbol.0;
            write!(f, "{symbol:>longest_var_length$} -> {value}")?;
        }
        Ok(())
    }
}

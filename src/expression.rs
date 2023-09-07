use anyhow::{anyhow, bail, Context, Result};
use std::{fmt::Display, iter::Peekable};

use crate::{atoms::*, errors::TypeError, token::Token, Environment};

pub trait LispExpression: Sized {
    fn as_atom(&self) -> &dyn Atom;

    fn as_list(&self) -> std::result::Result<&List<Self>, TypeError>;

    fn is_truthy(&self) -> bool {
        self.as_list().map(|l| l.0.is_empty()).unwrap_or(false)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Symbol(Symbol),
    Number(Number),
    List(List<Expression>),
    BuiltinFunction(BuiltinFunction<Expression>),
    BuiltinMacro(BuiltinMacro<Expression>),
    Lambda(Lambda<Expression>),
    Macro(Macro<Expression>),
}

impl LispExpression for Expression {
    fn as_atom(&self) -> &dyn Atom {
        match self {
            Expression::Symbol(a) => a,
            Expression::Number(a) => a,
            Expression::List(a) => a,
            Expression::BuiltinFunction(a) => a,
            Expression::BuiltinMacro(a) => a,
            Expression::Lambda(a) => a,
            Expression::Macro(a) => a,
        }
    }

    fn as_list(&self) -> std::result::Result<&List<Self>, TypeError>
    where
        Self: Sized,
    {
        self.try_into()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_atom().fmt(f)
    }
}

impl Expression {
    pub fn parse<I>(tokens: &mut Peekable<I>) -> Result<Expression>
    where
        I: Iterator<Item = Token>,
    {
        match tokens.next() {
            Some(token) if token.value == "(" => {
                let mut expressions = Vec::new();
                while !matches!(tokens.peek(), Some(token) if token.value == ")") {
                    expressions.push(Expression::parse(tokens).with_context(|| {
                        format!("While parsing list that began at {}", token.position)
                    })?);
                }
                tokens.next();
                let mut quoted_expressions = Vec::new();
                let mut expressions = expressions.into_iter().peekable();
                while let Some(expr) = expressions.next() {
                    if expr == Symbol("'".to_owned()).into() {
                        if let Some(next) = expressions.next() {
                            quoted_expressions.push(List(vec![expr, next]).into())
                        } else {
                            bail!("Trailing quote in input")
                        }
                    } else {
                        quoted_expressions.push(expr);
                    }
                }
                Ok(List(quoted_expressions).into())
            }
            Some(token) if token.value == ")" => {
                bail!("Unexpected close bracket at {}", token.position)
            }
            Some(token) => {
                if let Ok(value_as_float) = token.value.parse() {
                    Ok(Expression::Number(Number(value_as_float)))
                } else {
                    Ok(Expression::Symbol(Symbol(token.value.clone())))
                }
            }
            None => bail!("Ran out of tokens"),
        }
    }

    fn variant(&self) -> &'static str {
        match self {
            Expression::Symbol(_) => "symbol",
            Expression::Number(_) => "number",
            Expression::List(_) => "list",
            Expression::BuiltinFunction(_) => "builtin function",
            Expression::BuiltinMacro(_) => "builtin macro",
            Expression::Lambda(_) => "lambda",
            Expression::Macro(_) => "macro",
        }
    }

    pub fn eval(&self, env: &mut Environment<Self>) -> Result<Self> {
        match self {
            Expression::List(list) => {
                let function: Expression = list
                    .0
                    .get(0)
                    .ok_or_else(|| anyhow!("Attempt to evaluate empty list"))
                    .and_then(|e| e.eval(env))
                    .with_context(|| anyhow!("Could not evaluate head of list"))?;
                function.as_atom().call(&list.0[1..], env)
            }
            Expression::Symbol(symbol) => env
                .get(symbol)
                .cloned()
                .ok_or_else(|| anyhow!("Variable `{}` unbound", symbol)),
            expression => Ok(expression.clone()),
        }
    }
}

macro_rules! impl_try_to_from {
    ($type:tt, $atom:tt$(<$g:tt>)?) => {
        impl From<$atom$(<$g>)?> for $type {
            fn from(value: $atom$(<$g>)?) -> Self {
                $type::$atom(value)
            }
        }

        impl<'a> TryFrom<&'a $type> for &'a $atom$(<$g>)? {
            type Error = TypeError;

            fn try_from(value: &'a $type) -> std::result::Result<Self, Self::Error> {
                match value {
                    $type::$atom(inner) => Ok(inner),
                    _ => Err(TypeError {
                        expected: "???",
                        got: value.variant(),
                    }),
                }
            }
        }
    };
    ($type:tt, $($atom:tt$(<$g:tt>)?),+) => {
      $(
         impl_try_to_from!($type, $atom$(<$g>)?);
       )+
    };
}

impl_try_to_from!(
    Expression,
    Symbol,
    Number,
    Lambda<Expression>,
    Macro<Expression>,
    BuiltinFunction<Expression>,
    BuiltinMacro<Expression>,
    List<Expression>
);

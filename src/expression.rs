use anyhow::{anyhow, bail, Context, Result};
use std::{fmt::Display, iter::Peekable};

use crate::{atoms::*, errors::TypeError, token::Token, Environment};

pub trait ToAndFrom<T>: From<T> {
    fn try_into_atom(&self) -> std::result::Result<&T, TypeError>;
}

pub trait LispExpression:
    // TODO Can we implement display ourselves?
    Sized + Clone + PartialEq + Display + ToAndFrom<List<Self>> + ToAndFrom<Symbol> + ToAndFrom<Lambda<Self>> + ToAndFrom<Macro<Self>> + ToAndFrom<Number>
{
    fn as_atom(&self) -> &dyn Atom<Self>;

    fn as_list(&self) -> std::result::Result<&List<Self>, TypeError>;

    fn is_truthy(&self) -> bool {
        self.as_list().map(|l| l.0.is_empty()).unwrap_or(false)
    }

    fn parse_from_token(token: &Token) -> Self;

    fn variant(&self) -> &'static str {
        self.as_atom().name()
    }

    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        match tokens.next() {
            Some(token) if token.value == "(" => {
                let mut expressions = Vec::new();
                while !matches!(tokens.peek(), Some(token) if token.value == ")") {
                    expressions.push(Self::parse(tokens).with_context(|| {
                        format!("While parsing list that began at {}", token.position)
                    })?);
                }
                tokens.next();
                let mut quoted_expressions = Vec::new();
                let mut expressions = expressions.into_iter().peekable();
                while let Some(expr) = expressions.next() {
                    // TODO Look at this methodology
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
            Some(token) => Ok(Self::parse_from_token(&token)),
            None => bail!("Ran out of tokens"),
        }
    }

    fn eval(&self, env: &mut Environment<Self>) -> Result<Self> {
        if let Ok(list) = <Self as ToAndFrom<List<_>>>::try_into_atom(self) {
            let function: Self = list
                .0
                .get(0)
                .ok_or_else(|| anyhow!("Attempt to evaluate empty list"))
                .and_then(|e| e.eval(env))
                .with_context(|| anyhow!("Could not evaluate head of list"))?;
            function.as_atom().call(&list.0[1..], env)
        } else if let Ok(symbol) = <Self as ToAndFrom<Symbol>>::try_into_atom(self) {
            env.get(symbol)
                .cloned()
                .ok_or_else(|| anyhow!("Variable `{}` unbound", symbol))
        } else {
            Ok(self.clone())
        }
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
    fn as_atom(&self) -> &dyn Atom<Self> {
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
        self.try_into_atom()
    }

    fn parse_from_token(token: &Token) -> Self {
        None.or_else(|| <Number as Atom<Self>>::parse_from_token(token).map(Self::from))
            .or_else(|| <Symbol as Atom<Self>>::parse_from_token(token).map(Self::from))
            // This will never fail as symbols never fail parsing
            .unwrap()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_atom().fmt(f)
    }
}

macro_rules! impl_try_to_from {
    ($type:tt, $atom:tt$(<$g:tt>)?) => {
        impl From<$atom$(<$g>)?> for $type {
            fn from(value: $atom$(<$g>)?) -> Self {
                $type::$atom(value)
            }
        }

        impl ToAndFrom<$atom$(<$g>)?> for $type {
            fn try_into_atom(&self) -> std::result::Result<&$atom$(<$g>)?, TypeError> {
                match self {
                    $type::$atom(inner) => Ok(inner),
                    _ => Err(TypeError {
                        expected: <$atom$(<$g>)? as Atom<Self>>::sized_name(),
                        got: self.variant(),
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

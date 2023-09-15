use anyhow::{anyhow, bail, Context, Result};
use std::{fmt::Display, iter::Peekable};

use crate::{atoms::*, errors::TypeError, token::Token, Environment};

pub trait ToAndFrom<T>: From<T> {
    fn try_into_atom(&self) -> std::result::Result<&T, TypeError>;
}

pub trait LispExpression:
    'static
    + Sized
    + Clone
    + PartialEq
    + Display
    + ToAndFrom<List<Self>>
    + ToAndFrom<Symbol>
    + ToAndFrom<Lambda<Self>>
    + ToAndFrom<Macro<Self>>
    + ToAndFrom<BuiltinFunction<Self>>
    + ToAndFrom<BuiltinMacro<Self>>
    + ToAndFrom<Number>
{
    fn as_atom(&self) -> &dyn Atom<Self>;

    fn null() -> Self {
        List(vec![]).into()
    }

    fn as_list(&self) -> std::result::Result<&List<Self>, TypeError> {
        self.try_into_atom()
    }

    fn as_symbol(&self) -> std::result::Result<&Symbol, TypeError> {
        self.try_into_atom()
    }

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
        if let Ok(list) = self.as_list() {
            let function: Self = list
                .0
                .get(0)
                .ok_or_else(|| anyhow!("Attempt to evaluate empty list"))
                .and_then(|e| e.eval(env))
                .with_context(|| anyhow!("Could not evaluate head of list"))?;
            function.as_atom().call(&list.0[1..], env)
        } else if let Ok(symbol) = self.as_symbol() {
            env.get(symbol)
                .cloned()
                .ok_or_else(|| anyhow!("Variable `{}` unbound", symbol))
        } else {
            Ok(self.clone())
        }
    }
}

#[macro_export]
macro_rules! create_expression {
    ($expression_name:ident, $($atom:tt$(<$g:tt>)?),+) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum $expression_name {
            $(
            $atom($atom$(<$g>)?)
            ),+
        }


        impl std::fmt::Display for $expression_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.as_atom().fmt(f)
            }
        }

        impl LispExpression for $expression_name {
            fn as_atom(&self) -> &dyn Atom<Self> {
                match self {
                    $(
                    $expression_name::$atom(a) => a,
                    )*
                }
            }

            fn parse_from_token(token: &Token) -> Self {
                None
                    $(
                    .or_else(|| <$atom$(<$g>)? as Atom<Self>>::parse_from_token(token).map(Self::from))
                    )*
                    // This will never fail as symbols never fail parsing
                    .unwrap()
            }
        }

        $(
        impl From<$atom$(<$g>)?> for $expression_name {
            fn from(value: $atom$(<$g>)?) -> Self {
                $expression_name::$atom(value)
            }
        }

        impl ToAndFrom<$atom$(<$g>)?> for $expression_name {
            fn try_into_atom(&self) -> std::result::Result<&$atom$(<$g>)?, TypeError> {
                match self {
                    $expression_name::$atom(inner) => Ok(inner),
                    _ => Err(TypeError {
                        expected: <$atom$(<$g>)? as Atom<Self>>::sized_name(),
                        got: self.variant(),
                    }),
                }
            }

        }
        )?
    };
}

create_expression!(
    Expression,
    Symbol,
    Number,
    Lambda<Expression>,
    Macro<Expression>,
    BuiltinFunction<Expression>,
    BuiltinMacro<Expression>,
    List<Expression>
);

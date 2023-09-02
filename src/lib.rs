use anyhow::{anyhow, bail, Context, Result};
use std::{fmt::Display, iter::Peekable};

mod token;
pub use token::tokenize;
use token::*;
mod errors;
use errors::*;
mod builtins;

#[derive(Clone, Debug)]
pub enum Expression {
    Symbol(String),
    Number(f64),
    List(Vec<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Symbol(symbol) => {
                write!(f, "\x1b[0;32m{}\x1b[0m", symbol)
            }
            Expression::Number(number) => {
                write!(f, "\x1b[0;36m{}\x1b[0m", number)
            }
            Expression::List(list) => {
                let elements: Vec<String> = list.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", elements.join(" "))
            }
        }
    }
}

impl TryFrom<&Expression> for f64 {
    type Error = TypeError;

    fn try_from(value: &Expression) -> std::result::Result<Self, Self::Error> {
        match value {
            Expression::Number(number) => Ok(*number),
            _ => Err(TypeError {
                expected: "Number",
                got: value.variant(),
            }),
        }
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
                Ok(Expression::List(expressions))
            }
            Some(token) if token.value == ")" => {
                bail!("Unexpected close bracket at {}", token.position)
            }
            Some(token) => {
                if let Ok(value_as_float) = token.value.parse() {
                    Ok(Expression::Number(value_as_float))
                } else {
                    Ok(Expression::Symbol(token.value.clone()))
                }
            }
            None => bail!("Ran out of tokens"),
        }
    }

    fn variant(&self) -> &'static str {
        match self {
            Expression::Symbol(_) => "Symbol",
            Expression::Number(_) => "Number",
            Expression::List(_) => "List",
        }
    }

    pub fn eval(&self) -> Result<Expression> {
        match self {
            Expression::List(list) => {
                let function: Expression = list
                    .get(0)
                    .ok_or_else(|| anyhow!("Attempt to evaluate empty list"))
                    .and_then(|e| e.eval())
                    .with_context(|| anyhow!("Could not evaluate head of list"))?;
                let arguments: Vec<Expression> = list[1..]
                    .iter()
                    .enumerate()
                    .map(|(n, e)| {
                        e.eval()
                            .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
                    })
                    .collect::<Result<Vec<_>>>()
                    .with_context(|| anyhow!("Could not evaluate arguments to {:?}", function))?;
                match function {
                    Expression::Symbol(s) if s == "+" => builtins::add(&arguments),
                    Expression::Symbol(s) if s == "*" => builtins::mul(&arguments),
                    Expression::Symbol(s) if s == "list" => builtins::list(&arguments),
                    _ => {
                        bail!("Cannot call {:?} as a function", function)
                    }
                }
            }
            expression => Ok(expression.clone()),
        }
    }
}

pub fn evaluate(input: &str) -> Result<Expression> {
    let mut tokens = tokenize(&input).peekable();
    let expression = Expression::parse(&mut tokens)
        .with_context(|| anyhow!("Could not parse input {}", input))?;
    if tokens.peek().is_some() {
        bail!("Extra tokens in line")
    }
    expression
        .eval()
        .with_context(|| anyhow!("Could not evaluate input {}", input))
}

use anyhow::{anyhow, bail, Context, Result};
pub use atoms::*;
use std::{fmt::Display, iter::Peekable};

mod atoms;
mod token;
pub use token::tokenize;
use token::*;
mod errors;
use errors::*;
pub mod builtins;
mod environment;
pub use environment::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Symbol(Symbol),
    Number(f64),
    List(Vec<Expression>),
    BuiltinFunction(BuiltinFunction<Expression>),
    BuiltinMacro(BuiltinMacro<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Symbol(symbol) => {
                write!(f, "{}", symbol)
            }
            Expression::BuiltinFunction(builtin) => {
                write!(f, "{}", builtin)
            }
            Expression::BuiltinMacro(macr) => {
                write!(f, "{}", macr)
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
                let mut quoted_expressions = Vec::new();
                let mut expressions = expressions.into_iter().peekable();
                while let Some(expr) = expressions.next() {
                    if expr == Symbol("'".to_owned()).into() {
                        if let Some(next) = expressions.next() {
                            quoted_expressions.push(Expression::List(vec![expr, next]))
                        } else {
                            bail!("Trailing quote in input")
                        }
                    } else {
                        quoted_expressions.push(expr);
                    }
                }
                Ok(Expression::List(quoted_expressions))
            }
            Some(token) if token.value == ")" => {
                bail!("Unexpected close bracket at {}", token.position)
            }
            Some(token) => {
                if let Ok(value_as_float) = token.value.parse() {
                    Ok(Expression::Number(value_as_float))
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
        }
    }

    pub fn eval(&self, env: &mut Environment<Self>) -> Result<Self> {
        match self {
            Expression::List(list) => {
                let function: Expression = list
                    .get(0)
                    .ok_or_else(|| anyhow!("Attempt to evaluate empty list"))
                    .and_then(|e| e.eval(env))
                    .with_context(|| anyhow!("Could not evaluate head of list"))?;
                match function {
                    Expression::BuiltinFunction(func) => {
                        let arguments: Vec<Expression> = list[1..]
                            .iter()
                            .enumerate()
                            .map(|(n, e)| {
                                e.eval(env)
                                    .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
                            })
                            .collect::<Result<Vec<_>>>()
                            .with_context(|| {
                                anyhow!("Could not evaluate arguments to {:?}", func)
                            })?;
                        func.0(&arguments, env)
                    }
                    Expression::BuiltinMacro(func) => func.0(&list[1..], env),
                    _ => {
                        bail!("Cannot call {:?} as a function", function)
                    }
                }
            }
            Expression::Symbol(symbol) => env
                .get(symbol)
                .cloned()
                .ok_or_else(|| anyhow!("Variable `{}` unbound", symbol)),
            expression => Ok(expression.clone()),
        }
    }
}

impl From<Symbol> for Expression {
    fn from(value: Symbol) -> Self {
        Expression::Symbol(value)
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Self {
        Expression::Number(value)
    }
}

impl From<BuiltinFunction<Expression>> for Expression {
    fn from(value: BuiltinFunction<Expression>) -> Self {
        Expression::BuiltinFunction(value)
    }
}

impl From<BuiltinMacro<Expression>> for Expression {
    fn from(value: BuiltinMacro<Expression>) -> Self {
        Expression::BuiltinMacro(value)
    }
}

pub fn evaluate(input: &str, env: &mut Environment<Expression>) -> Result<Expression> {
    let mut tokens = tokenize(&input).peekable();
    let expression = Expression::parse(&mut tokens)
        .with_context(|| anyhow!("Could not parse input {}", input))?;
    if tokens.peek().is_some() {
        bail!("Extra tokens in line")
    }
    expression
        .eval(env)
        .with_context(|| anyhow!("Could not evaluate input {}", input))
}

use anyhow::{anyhow, bail, Context, Result};
use std::iter::Peekable;

mod token;
use token::*;
mod errors;
use errors::*;

#[derive(Clone, Debug)]
enum Expression {
    Symbol(String),
    Number(f64),
    List(Vec<Expression>),
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
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Expression>
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

    fn eval(&self) -> Result<Expression> {
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
                    Expression::Symbol(s) if s == "+" => {
                        let arguments: Vec<f64> = arguments
                            .iter()
                            .enumerate()
                            .map(|(n, e)| {
                                e.try_into()
                                    .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
                            })
                            .collect::<std::result::Result<Vec<f64>, _>>()
                            .context("Arguments to add are not all numbers")?;
                        Ok(Expression::Number(arguments.iter().sum()))
                    }
                    Expression::Symbol(s) if s == "*" => {
                        let arguments: Vec<f64> = arguments
                            .iter()
                            .enumerate()
                            .map(|(n, e)| {
                                e.try_into()
                                    .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
                            })
                            .collect::<std::result::Result<Vec<f64>, _>>()
                            .context("Arguments to mul are not all numbers")?;
                        Ok(Expression::Number(arguments.iter().product()))
                    }
                    _ => {
                        bail!("Cannot call {:?} as a function", function)
                    }
                }
            }
            expression => Ok(expression.clone()),
        }
    }
}

fn main() -> Result<()> {
    let program = r#"(+ 1 (* 2 3) 4.3)"#;
    let mut tokens = tokenize(program).peekable();
    let expression: Expression = Expression::parse(&mut tokens)?;
    let result = expression
        .eval()
        .context("While evaluating root expression")?;
    dbg!(result);
    Ok(())
}

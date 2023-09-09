use anyhow::{anyhow, bail, Context, Result};

mod atoms;
pub use atoms::*;
mod token;
pub use token::tokenize;
pub mod builtins;
mod environment;
mod errors;
pub use environment::*;
mod expression;
pub use expression::{Expression, LispExpression};

// TODO Symbol interning?

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

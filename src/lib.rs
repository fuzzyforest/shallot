use anyhow::{anyhow, bail, Context, Result};

mod atoms;
pub use atoms::*;
mod token;
pub use token::{tokenize, Token};
pub mod builtins;
pub use builtins::set_environment;
mod environment;
mod errors;
pub use environment::*;
pub use errors::TypeError;
mod expression;
pub use expression::{Expression, LispExpression, ToAndFrom};

mod repl;
pub use repl::run_repl;

// TODO Symbol interning?

pub fn evaluate<E: LispExpression>(input: &str, env: &mut Environment<E>) -> Result<E> {
    let mut tokens = tokenize(input).peekable();
    let expression =
        E::parse(&mut tokens).with_context(|| anyhow!("Could not parse input {}", input))?;
    if tokens.peek().is_some() {
        bail!("Extra tokens in line")
    }
    expression
        .eval(env)
        .with_context(|| anyhow!("Could not evaluate input {}", input))
}

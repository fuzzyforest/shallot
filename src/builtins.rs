use crate::{Environment, Expression};
use anyhow::{anyhow, bail, Context, Result};

fn expressions_to_floats(expressions: &[Expression]) -> Result<Vec<f64>> {
    expressions
        .iter()
        .enumerate()
        .map(|(n, e)| {
            e.try_into()
                .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
        })
        .collect::<std::result::Result<Vec<f64>, _>>()
}

pub fn add(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to add are not all numbers")?;
    Ok(Expression::Number(arguments.iter().sum()))
}

pub fn sub(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to add are not all numbers")?;
    if let Some(first) = arguments.get(0) {
        Ok(Expression::Number(
            first - arguments[1..].iter().sum::<f64>(),
        ))
    } else {
        bail!("Insufficient arguments to sub")
    }
}

pub fn mul(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to mul are not all numbers")?;
    Ok(Expression::Number(arguments.iter().product()))
}

pub fn div(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to add are not all numbers")?;
    if let Some(first) = arguments.get(0) {
        Ok(Expression::Number(
            first / arguments[1..].iter().product::<f64>(),
        ))
    } else {
        bail!("Insufficient arguments to div")
    }
}

pub fn eq(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    if let Some(first) = arguments.get(0) {
        let mut last = first;
        for elt in arguments[1..].iter() {
            if elt != last {
                return Ok(Expression::List(vec![]));
            }
            last = elt;
        }
        return Ok(Expression::Number(1.));
    } else {
        return Ok(Expression::Number(1.));
    }
}

pub fn list(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    Ok(Expression::List(arguments.to_vec()))
}

pub fn define(arguments: &[Expression], env: &mut Environment<Expression>) -> Result<Expression> {
    if arguments.len() != 2 {
        bail!("Define requires two arguments")
    }
    if let Expression::Symbol(symbol) = &arguments[0] {
        env.set(symbol.clone(), arguments[1].clone());
        // This will never be None because we just set it
        env.get(symbol).cloned().ok_or_else(|| unreachable!())
    } else {
        bail!("First argument to define should be a symbol")
    }
}

pub fn quote(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    if arguments.len() != 1 {
        bail!("Quote must be called on exactly one argument")
    }
    Ok(arguments[0].clone())
}

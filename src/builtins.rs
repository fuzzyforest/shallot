use crate::Expression;
use anyhow::{anyhow, Context, Result};

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

pub fn add(arguments: &[Expression]) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to add are not all numbers")?;
    Ok(Expression::Number(arguments.iter().sum()))
}

pub fn mul(arguments: &[Expression]) -> Result<Expression> {
    let arguments =
        expressions_to_floats(arguments).context("Arguments to mul are not all numbers")?;
    Ok(Expression::Number(arguments.iter().product()))
}

pub fn list(arguments: &[Expression]) -> Result<Expression> {
    Ok(Expression::List(arguments.to_vec()))
}

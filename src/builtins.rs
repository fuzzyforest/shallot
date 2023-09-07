use crate::{
    errors::TypeError, expression::LispExpression, Environment, Expression, Lambda, List, Macro,
    Number, Symbol,
};
use anyhow::{anyhow, bail, Context, Result};

fn expressions_to_homogeneous<'a, T>(expressions: &'a [Expression]) -> Result<Vec<T>>
where
    T: TryFrom<&'a Expression, Error = TypeError>,
{
    expressions
        .iter()
        .enumerate()
        .map(|(n, e)| {
            e.try_into()
                .with_context(|| anyhow!("Argument number {}: {:?}", n + 1, e))
        })
        .collect::<std::result::Result<Vec<T>, _>>()
}

pub fn le(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    for i in 0..arguments.len() - 1 {
        if !(arguments[i] <= arguments[i + 1]) {
            return Ok(List(vec![]).into());
        }
    }
    Ok(Number(1.).into())
}

pub fn add(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    Ok(Number(arguments.iter().sum()).into())
}

pub fn sub(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    if let Some(first) = arguments.get(0) {
        Ok(Number(first - arguments[1..].iter().sum::<f64>()).into())
    } else {
        bail!("Insufficient arguments to sub")
    }
}

pub fn mul(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to mul are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    Ok(Number(arguments.iter().product()).into())
}

pub fn div(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    if let Some(first) = arguments.get(0) {
        Ok(Number(first / arguments[1..].iter().product::<f64>()).into())
    } else {
        bail!("Insufficient arguments to div")
    }
}

pub fn eq(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    if let Some(first) = arguments.get(0) {
        let mut last = first;
        for elt in arguments[1..].iter() {
            if elt != last {
                return Ok(List(vec![]).into());
            }
            last = elt;
        }
        return Ok(Number(1.).into());
    } else {
        return Ok(Number(1.).into());
    }
}

pub fn list(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    Ok(List(arguments.to_vec()).into())
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

pub fn lambda(arguments: &[Expression], env: &mut Environment<Expression>) -> Result<Expression> {
    if arguments.len() != 2 {
        bail!("Lambdas must be constructed with exactly two arguments");
    }
    let Expression::List(parameters) = &arguments[0] else {
        bail!("First argument to lambda construction must be a list")
    };
    let parameters: Vec<&Symbol> = expressions_to_homogeneous(&parameters.0)
        .context("Parameter names need to all be symbols")?;
    let value = arguments[1].clone();
    Ok(Lambda {
        parameters: parameters.into_iter().cloned().collect(),
        value: Box::new(value),
        env: env.clone(),
    }
    .into())
}

pub fn macr(arguments: &[Expression], env: &mut Environment<Expression>) -> Result<Expression> {
    if arguments.len() != 2 {
        bail!("Macros must be constructed with exactly two arguments");
    }
    let Expression::List(parameters) = &arguments[0] else {
        bail!("First argument to macros construction must be a list")
    };
    let parameters: Vec<&Symbol> = expressions_to_homogeneous(&parameters.0)
        .context("Parameter names need to all be symbols")?;
    let value = arguments[1].clone();
    Ok(Macro {
        parameters: parameters.into_iter().cloned().collect(),
        value: Box::new(value),
        env: env.clone(),
    }
    .into())
}

pub fn cond(arguments: &[Expression], env: &mut Environment<Expression>) -> Result<Expression> {
    for i in 0..arguments.len() / 2 {
        let condition_number = i + 1;
        if arguments[2 * i]
            .eval(env)
            .with_context(|| anyhow!("Could not evalutate condition number {condition_number}"))?
            .is_truthy()
        {
            return arguments[2 * i + 1].eval(env).with_context(|| {
                anyhow!("Could not evaluate consequence number {condition_number}")
            });
        }
    }
    if arguments.len() % 2 == 0 {
        Ok(List(vec![]).into())
    } else {
        arguments
            .last()
            .unwrap() // Length is odd
            .eval(env)
            .context("Could not evaluate default")
    }
}

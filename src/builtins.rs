use crate::{
    expression::{LispExpression, ToAndFrom},
    BuiltinFunction, BuiltinMacro, Environment, Lambda, List, Macro, Number, Symbol,
};
use anyhow::{anyhow, bail, Context, Result};

fn expressions_to_homogeneous<E, T>(expressions: &[E]) -> Result<Vec<&T>>
where
    E: LispExpression + ToAndFrom<T>,
{
    expressions
        .iter()
        .enumerate()
        .map(|(n, e)| {
            e.try_into_atom()
                .with_context(|| anyhow!("Argument number {}: {}", n + 1, e))
        })
        .collect()
}

pub fn le<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression + ToAndFrom<Number>,
{
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    for i in 0..arguments.len() - 1 {
        if arguments[i] > arguments[i + 1] {
            return Ok(E::null());
        }
    }
    Ok(Number(1.).into())
}

pub fn add<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression + ToAndFrom<Number>,
{
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    Ok(Number(arguments.iter().sum()).into())
}

pub fn sub<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression + ToAndFrom<Number>,
{
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    if let Some(first) = arguments.first() {
        Ok(Number(first - arguments[1..].iter().sum::<f64>()).into())
    } else {
        bail!("Insufficient arguments to sub")
    }
}

pub fn mul<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression + ToAndFrom<Number>,
{
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to mul are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    Ok(Number(arguments.iter().product()).into())
}

pub fn div<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression + ToAndFrom<Number>,
{
    let arguments: Vec<&Number> =
        expressions_to_homogeneous(arguments).context("Arguments to add are not all numbers")?;
    let arguments: Vec<f64> = arguments.into_iter().map(|n| n.0).collect();
    if let Some(first) = arguments.first() {
        Ok(Number(first / arguments[1..].iter().product::<f64>()).into())
    } else {
        bail!("Insufficient arguments to div")
    }
}

pub fn eq<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    if let Some(first) = arguments.first() {
        let mut last = first;
        for elt in arguments[1..].iter() {
            if elt != last {
                return Ok(List(vec![]).into());
            }
            last = elt;
        }
        Ok(Number(1.).into())
    } else {
        Ok(Number(1.).into())
    }
}

pub fn list<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    Ok(List(arguments.to_vec()).into())
}

pub fn define<E>(arguments: &[E], env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    if arguments.len() != 2 {
        bail!("Define requires two arguments")
    }
    let symbol: &Symbol = arguments[0]
        .try_into_atom()
        .context("First argument to define should be a symbol")?;
    env.set(symbol.clone(), arguments[1].clone());
    // This will never be None because we just set it
    env.get(symbol).cloned().ok_or_else(|| unreachable!())
}

pub fn quote<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    if arguments.len() != 1 {
        bail!("Quote must be called on exactly one argument")
    }
    Ok(arguments[0].clone())
}

pub fn lambda<E>(arguments: &[E], env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    if arguments.len() != 2 {
        bail!("Lambdas must be constructed with exactly two arguments");
    }
    let parameters: &List<_> = arguments[0]
        .try_into_atom()
        .context("First argument to lambda construction must be a list")?;
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

pub fn macr<E>(arguments: &[E], env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    if arguments.len() != 2 {
        bail!("Macros must be constructed with exactly two arguments");
    }
    let parameters: &List<_> = arguments[0]
        .try_into_atom()
        .context("First argument to macros construction must be a list")?;
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

pub fn cond<E>(arguments: &[E], env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
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

pub fn set_environment<E: LispExpression + ToAndFrom<Number>>(env: &mut Environment<E>) {
    env.set("≤", BuiltinFunction::new("≤", le));
    env.set("cond", BuiltinMacro::new("cond", cond));
    env.set("+", BuiltinFunction::new("+", add));
    env.set("*", BuiltinFunction::new("*", mul));
    env.set("-", BuiltinFunction::new("-", sub));
    env.set("/", BuiltinFunction::new("/", div));
    env.set("list", BuiltinFunction::new("list", list));
    env.set("=", BuiltinFunction::new("=", eq));
    env.set("define", BuiltinFunction::new("define", define));
    env.set("'", BuiltinMacro::new("'", quote));
    env.set("λ", BuiltinMacro::new("λ", lambda));
    env.set("μ", BuiltinMacro::new("μ", macr));
}

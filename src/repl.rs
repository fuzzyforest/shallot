use std::io::{Read, Write};
use std::path::PathBuf;

use crate::*;
use anyhow::{anyhow, Context, Result};

#[derive(Debug, Default)]
struct Arguments {
    path: Option<PathBuf>,
    interactive: bool,
}

fn get_arguments() -> Arguments {
    // TODO Help message
    // TODO Errors if incorrect arguments
    let mut arguments = Arguments::default();
    for argument in std::env::args().skip(1) {
        if argument == "-i" {
            arguments.interactive = true;
        } else {
            arguments.path = Some(argument.into());
        }
    }
    if arguments.path.is_none() {
        arguments.interactive = true;
    }
    arguments
}

fn print<E>(arguments: &[E], _env: &mut Environment<E>) -> Result<E>
where
    E: LispExpression,
{
    for argument in arguments {
        println!("{argument}");
    }
    Ok(List(vec![]).into())
}

pub fn run_repl<E>(environment: &mut Environment<E>) -> Result<()>
where
    E: LispExpression,
{
    let arguments = get_arguments();
    environment.set(
        Symbol("print".to_owned()),
        BuiltinFunction::new("print", print),
    );

    if let Some(path) = arguments.path {
        let input = match path.to_str() {
            Some("-") => {
                let mut input = String::new();
                std::io::stdin()
                    .read_to_string(&mut input)
                    .context("Could not read line")?;
                input
            }
            _ => std::fs::read_to_string(&path).with_context(|| {
                anyhow!(
                    "Could not read from {}",
                    path.to_str().unwrap_or("<Non-UTF8-Path>")
                )
            })?,
        };

        let result = evaluate(&input, environment)?;

        println!("{}", result);
    }
    if arguments.interactive {
        'repl: loop {
            print!("🧅 ");
            std::io::stdout()
                .flush()
                .context("Could not flush prompt")?;
            let mut input_line = String::new();
            std::io::stdin()
                .read_line(&mut input_line)
                .context("Could not read line")?;
            if input_line.is_empty() {
                break 'repl;
            }
            if input_line.chars().all(|c| c.is_whitespace()) {
                continue;
            }
            if input_line == "#env\n" {
                println!("{environment}");
                continue 'repl;
            }
            let result = evaluate(&input_line, environment);
            match result {
                Ok(result) => println!("{result}"),
                Err(error) => println!("{error:?}"),
            }
        }
    }

    Ok(())
}

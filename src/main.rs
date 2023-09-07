use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use shallot::*;

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

fn print(arguments: &[Expression], _env: &mut Environment<Expression>) -> Result<Expression> {
    for argument in arguments {
        println!("{argument}");
    }
    Ok(List(vec![]).into())
}

fn main() -> Result<()> {
    let arguments = get_arguments();
    let mut environment: Environment<Expression> = Environment::default();
    // Default bindings
    environment.set(Symbol("pi".to_owned()), Expression::from(Number(3.1415)));
    environment.set(
        Symbol("≤".to_owned()),
        BuiltinFunction {
            name: "≤",
            function: builtins::le,
        },
    );
    environment.set(
        Symbol("cond".to_owned()),
        BuiltinMacro {
            name: "cond",
            function: builtins::cond,
        },
    );
    environment.set(
        Symbol("print".to_owned()),
        BuiltinFunction {
            name: "print",
            function: print,
        },
    );
    environment.set(
        Symbol("+".to_owned()),
        BuiltinFunction {
            name: "+",
            function: builtins::add,
        },
    );
    environment.set(
        Symbol("*".to_owned()),
        BuiltinFunction {
            name: "*",
            function: builtins::mul,
        },
    );
    environment.set(
        Symbol("-".to_owned()),
        BuiltinFunction {
            name: "-",
            function: builtins::sub,
        },
    );
    environment.set(
        Symbol("/".to_owned()),
        BuiltinFunction {
            name: "/",
            function: builtins::div,
        },
    );
    environment.set(
        Symbol("list".to_owned()),
        BuiltinFunction {
            name: "list",
            function: builtins::list,
        },
    );
    environment.set(
        Symbol("=".to_owned()),
        BuiltinFunction {
            name: "=",
            function: builtins::eq,
        },
    );
    environment.set(
        Symbol("define".to_owned()),
        BuiltinFunction {
            name: "define",
            function: builtins::define,
        },
    );
    environment.set(
        Symbol("'".to_owned()),
        BuiltinMacro {
            name: "'",
            function: builtins::quote,
        },
    );
    environment.set(
        Symbol("λ".to_owned()),
        BuiltinMacro {
            name: "λ",
            function: builtins::lambda,
        },
    );
    environment.set(
        Symbol("μ".to_owned()),
        BuiltinMacro {
            name: "μ",
            function: builtins::macr,
        },
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

        let result = evaluate(&input, &mut environment)?;

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
            let result = evaluate(&input_line, &mut environment);
            match result {
                Ok(result) => println!("{result}"),
                Err(error) => println!("{error:?}"),
            }
        }
    }

    Ok(())
}

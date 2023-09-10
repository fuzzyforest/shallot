use anyhow::Result;
use shallot::*;

fn main() -> Result<()> {
    let mut environment: Environment<Expression> = Environment::default();
    shallot::builtins::set_environment(&mut environment);
    run_repl::<Expression>(&mut environment)
}

use std::{error::Error, fmt::Display};

#[derive(Copy, Clone, Debug)]
pub struct TypeError {
    pub expected: &'static str,
    pub got: &'static str,
}

impl Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type error: expected {} and got {}",
            self.expected, self.got
        )
    }
}

impl Error for TypeError {}

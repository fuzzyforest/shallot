use anyhow::{bail, Context, Result};
use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

#[derive(Debug)]
struct Token {
    value: String,
    position: usize,
}

struct TokenIterator<'a> {
    input: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.input.next_if(|c| c.1.is_whitespace()).is_some() {}

        // NOTE: The first character is not whitespace
        match self.input.peek() {
            Some((_, '(' | ')')) => self.input.next().map(|c| Token {
                value: c.1.into(),
                position: c.0,
            }),
            Some((position, ';')) => {
                let position = *position;
                let mut comment_token = String::new();
                while let Some(c) = self.input.next_if(|c| c.1 != '\n') {
                    comment_token.push(c.1)
                }
                Some(Token {
                    value: comment_token,
                    position,
                })
            }
            Some((position, '"')) => {
                let position = *position;
                let mut multi_word_token = "\"".to_owned();
                self.input.next();
                loop {
                    match self.input.peek() {
                        Some((_, '\\')) => {
                            self.input.next();
                            match self.input.peek() {
                                Some((_, '\"')) => {
                                    multi_word_token.push('"');
                                    self.input.next();
                                }
                                Some((_, '\\')) => {
                                    multi_word_token.push('\\');
                                    self.input.next();
                                }
                                Some((_, c)) => {
                                    multi_word_token.push('\\');
                                    multi_word_token.push(*c);
                                    self.input.next();
                                }
                                None => {
                                    multi_word_token.push('\\');
                                }
                            }
                        }
                        Some((_, '"')) => {
                            multi_word_token.push('"');
                            self.input.next();
                            break;
                        }
                        Some((_, c)) => {
                            multi_word_token.push(*c);
                            self.input.next();
                        }
                        None => break,
                    }
                }
                Some(Token {
                    value: multi_word_token,
                    position,
                })
            }
            Some((position, _)) => {
                let position = *position;
                let mut token = String::new();
                while let Some(c) = self
                    .input
                    .next_if(|c| !(c.1.is_whitespace() || "()".contains(c.1)))
                {
                    token.push(c.1)
                }
                Some(Token {
                    value: token,
                    position,
                })
            }
            None => None,
        }
    }
}

fn tokenize<'a>(input: &'a str) -> TokenIterator {
    TokenIterator {
        input: input.chars().enumerate().peekable(),
    }
}

#[derive(Debug)]
enum Expression {
    Atom(String),
    List(Vec<Expression>),
}

impl Expression {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Expression>
    where
        I: Iterator<Item = Token>,
    {
        match tokens.next() {
            Some(token) if token.value == "(" => {
                let mut expressions = Vec::new();
                while !matches!(tokens.peek(), Some(token) if token.value == ")") {
                    expressions.push(Expression::parse(tokens).with_context(|| {
                        format!("While parsing list that began at {}", token.position)
                    })?);
                }
                tokens.next();
                Ok(Expression::List(expressions))
            }
            Some(token) => return Ok(Expression::Atom(token.value.clone())),
            None => bail!("Ran out of tokens"),
        }
    }
}

fn main() -> Result<()> {
    let program = r#"(+ (* 2 3) 4)"#;
    let mut tokens = tokenize(program).peekable();
    dbg!(Expression::parse(&mut tokens)?);
    dbg!(tokens.collect::<Vec<_>>());
    Ok(())
}

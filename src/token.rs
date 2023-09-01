use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub position: usize,
}

pub struct TokenIterator<'a> {
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

pub fn tokenize<'a>(input: &'a str) -> TokenIterator {
    TokenIterator {
        input: input.chars().enumerate().peekable(),
    }
}

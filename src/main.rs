use std::{iter::Peekable, str::Chars};

struct TokenIterator<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.input.next_if(|c| c.is_whitespace()).is_some() {}

        // NOTE: The first character is not whitespace
        match self.input.peek() {
            Some('(') => {
                self.input.next();
                Some("(".to_owned())
            }
            Some(')') => {
                self.input.next();
                Some(")".to_owned())
            }
            Some(';') => {
                let mut comment_token = String::new();
                while let Some(c) = self.input.next_if(|c| *c != '\n') {
                    comment_token.push(c)
                }
                Some(comment_token)
            }
            // TODO: Parse tokens with whitespace delimited by ""
            Some(_) => {
                let mut token = String::new();
                while let Some(c) = self
                    .input
                    .next_if(|c| !(c.is_whitespace() || "()".contains(*c)))
                {
                    token.push(c)
                }
                Some(token)
            }
            None => None,
        }
    }
}

fn tokenize<'a>(input: &'a str) -> TokenIterator {
    TokenIterator {
        input: input.chars().peekable(),
    }
}

fn main() {
    let program = "(define double (lambda (x) (* x 2))) ; test comment\n (* 2 3)";
    let tokens: Vec<_> = tokenize(program).collect();
    dbg!(tokens);
}

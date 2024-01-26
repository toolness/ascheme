use std::{iter::Peekable, str::CharIndices};

pub struct Tokenizer<'a> {
    chars: Peekable<CharIndices<'a>>,
    curr_pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
}

type TokenRange = (usize, usize);

impl<'a> Tokenizer<'a> {
    pub fn new<T: AsRef<str>>(string: &'a T) -> Self {
        Tokenizer {
            chars: string.as_ref().char_indices().peekable(),
            curr_pos: 0,
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn chomp_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
        loop {
            if let Some(&(pos, next_char)) = self.chars.peek() {
                if !predicate(next_char) {
                    return;
                }
                self.chars.next();
                self.curr_pos = pos + next_char.len_utf8();
            } else {
                return;
            }
        }
    }

    fn accept_char(&mut self, char: char) -> bool {
        if let Some(&(pos, next_char)) = self.chars.peek() {
            if next_char == char {
                self.chars.next();
                self.curr_pos = pos + next_char.len_utf8();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn chomp_whitespace(&mut self) {
        self.chomp_while(|char| char.is_whitespace());
    }
}

pub type TokenWithRange = (Token, TokenRange);

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenWithRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp_whitespace();
        if self.is_at_end() {
            return None;
        }
        let token_start = self.curr_pos;
        if self.accept_char('(') {
            Some((Token::LeftParen, (token_start, self.curr_pos)))
        } else if self.accept_char(')') {
            Some((Token::RightParen, (token_start, self.curr_pos)))
        } else {
            todo!("Add support for more token types");
        }
    }
}

fn main() {
    let mut k = Tokenizer::new(&"  (  ) ");

    println!("Hello, world! {:?} {:?}", k.next(), k.next());
}

#[cfg(test)]
mod tests {
    use crate::Token::*;
    use crate::TokenWithRange;
    use crate::Tokenizer;

    fn test_tokenize_success(string: &'static str, expect: &[TokenWithRange]) {
        let tokenizer = Tokenizer::new(&string);
        let tokens = tokenizer.into_iter().collect::<Vec<_>>();
        assert_eq!(&tokens, expect, "Tokenization of '{}'", string);
    }

    #[test]
    fn parens_and_whitespace_works() {
        test_tokenize_success("  (  ) ", &[(LeftParen, (2, 3)), (RightParen, (5, 6))])
    }
}

use std::{iter::Peekable, str::CharIndices};

pub struct Tokenizer<'a> {
    chars: Peekable<CharIndices<'a>>,
    curr_pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Number,
    Identifier,
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
            if !self.accept(&predicate) {
                return;
            }
        }
    }

    fn accept<F: Fn(char) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(&(pos, next_char)) = self.chars.peek() {
            if predicate(next_char) {
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

    fn accept_char(&mut self, char: char) -> bool {
        self.accept(|next_char| next_char == char)
    }

    fn accept_number(&mut self) -> bool {
        let predicate = |char: char| char.is_numeric() || char == '.';
        if !self.accept(predicate) {
            return false;
        }
        self.chomp_while(predicate);
        true
    }

    fn accept_identifier(&mut self) -> bool {
        let is_ident_char =
            |char: char| !char.is_whitespace() && char != '.' && char != '(' && char != ')';
        if !self.accept(|char: char| !char.is_numeric() && is_ident_char(char)) {
            return false;
        }
        self.chomp_while(is_ident_char);
        true
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
        let token: Token = if self.accept_char('(') {
            Token::LeftParen
        } else if self.accept_char(')') {
            Token::RightParen
        } else if self.accept_number() {
            Token::Number
        } else if self.accept_identifier() {
            Token::Identifier
        } else {
            todo!("Add support for more token types");
        };
        Some((token, (token_start, self.curr_pos)))
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

    #[test]
    fn number_works() {
        test_tokenize_success(
            ".3 5.2 1",
            &[(Number, (0, 2)), (Number, (3, 6)), (Number, (7, 8))],
        )
    }

    #[test]
    fn identifier_works() {
        test_tokenize_success(
            "hi there? ",
            &[(Identifier, (0, 2)), (Identifier, (3, 9))],
        )
    }
}

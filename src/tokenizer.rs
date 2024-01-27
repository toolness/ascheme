use std::{iter::Peekable, str::CharIndices};

use crate::source_mapped::SourceMapped;

pub struct Tokenizer<'a> {
    chars: Peekable<CharIndices<'a>>,
    curr_pos: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    Number,
    Identifier,
}

pub type Token = SourceMapped<TokenType>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenizeErrorType {
    InvalidNumber,
    UnexpectedCharacter,
}

pub type TokenizeError = SourceMapped<TokenizeErrorType>;

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

    fn try_accept_number(&mut self) -> Option<Result<TokenType, TokenizeErrorType>> {
        let mut found_decimals = 0;
        let mut found_digit = false;
        loop {
            if self.accept_char('.') {
                found_decimals += 1;
            } else if self.accept(|char| char.is_numeric()) {
                found_digit = true;
            } else {
                break;
            }
        }
        if found_decimals > 1 {
            Some(Err(TokenizeErrorType::InvalidNumber))
        } else if found_digit {
            Some(Ok(TokenType::Number))
        } else {
            None
        }
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

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp_whitespace();
        if self.is_at_end() {
            return None;
        }
        let token_start = self.curr_pos;
        let token: Result<TokenType, TokenizeErrorType> = if self.accept_char('(') {
            Ok(TokenType::LeftParen)
        } else if self.accept_char(')') {
            Ok(TokenType::RightParen)
        } else if let Some(result) = self.try_accept_number() {
            result
        } else if self.accept_identifier() {
            Ok(TokenType::Identifier)
        } else {
            Err(TokenizeErrorType::UnexpectedCharacter)
        };
        let source = (token_start, self.curr_pos);
        Some(match token {
            Ok(token) => Ok(SourceMapped(token, source)),
            Err(error) => Err(SourceMapped(error, source)),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Tokenizer;

    use super::TokenType::{self, *};
    use super::TokenizeErrorType;

    fn test_tokenize(
        string: &'static str,
        expect: &[(Result<TokenType, TokenizeErrorType>, &'static str)],
    ) {
        let tokenizer = Tokenizer::new(&string);
        let tokens = tokenizer
            .into_iter()
            .map(|token| match token {
                Ok(token) => (Ok(token.0), token.source(string)),
                Err(err) => (Err(err.0), err.source(string)),
            })
            .collect::<Vec<_>>();
        assert_eq!(&tokens, expect, "Tokenization of '{string}'");
    }

    #[test]
    fn parens_and_whitespace_works() {
        test_tokenize("  (  ) ", &[(Ok(LeftParen), "("), (Ok(RightParen), ")")])
    }

    #[test]
    fn number_works() {
        test_tokenize(
            ".3 5.2 1 ..5",
            &[
                (Ok(Number), ".3"),
                (Ok(Number), "5.2"),
                (Ok(Number), "1"),
                (Err(TokenizeErrorType::InvalidNumber), "..5"),
            ],
        )
    }

    #[test]
    fn identifier_works() {
        test_tokenize(
            "hi there? ",
            &[(Ok(Identifier), "hi"), (Ok(Identifier), "there?")],
        )
    }
}

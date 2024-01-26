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

#[derive(Debug)]
pub enum TokenizeError {
    InvalidNumber,
    UnexpectedCharacter,
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

    fn try_accept_number(&mut self) -> Option<Result<Token, TokenizeError>> {
        let mut found_decimal = false;
        let mut found_digit = false;
        loop {
            if self.accept_char('.') {
                if found_decimal {
                    return Some(Err(TokenizeError::InvalidNumber));
                }
                found_decimal = true;
            } else if self.accept(|char| char.is_numeric()) {
                found_digit = true;
            } else {
                break;
            }
        }
        if found_digit {
            Some(Ok(Token::Number))
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

pub type TokenWithRange = (Result<Token, TokenizeError>, TokenRange);

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenWithRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp_whitespace();
        if self.is_at_end() {
            return None;
        }
        let token_start = self.curr_pos;
        let token: Result<Token, TokenizeError> = if self.accept_char('(') {
            Ok(Token::LeftParen)
        } else if self.accept_char(')') {
            Ok(Token::RightParen)
        } else if let Some(result) = self.try_accept_number() {
            result
        } else if self.accept_identifier() {
            Ok(Token::Identifier)
        } else {
            Err(TokenizeError::UnexpectedCharacter)
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
    use crate::Token;
    use crate::Token::*;
    use crate::Tokenizer;

    fn test_tokenize_success(string: &'static str, expect: &[(Token, &'static str)]) {
        let tokenizer = Tokenizer::new(&string);
        let tokens = tokenizer
            .into_iter()
            .map(|(token, range)| {
                (
                    token.unwrap_or_else(|err| panic!("Got {err:?} when tokenizing '{string}'")),
                    &string[range.0..range.1],
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(&tokens, expect, "Tokenization of '{string}'");
    }

    #[test]
    fn parens_and_whitespace_works() {
        test_tokenize_success("  (  ) ", &[(LeftParen, "("), (RightParen, ")")])
    }

    #[test]
    fn number_works() {
        test_tokenize_success(
            ".3 5.2 1",
            &[(Number, ".3"), (Number, "5.2"), (Number, "1")],
        )
    }

    #[test]
    fn identifier_works() {
        test_tokenize_success("hi there? ", &[(Identifier, "hi"), (Identifier, "there?")])
    }
}

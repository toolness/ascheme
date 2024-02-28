use std::{iter::Peekable, str::CharIndices};

use crate::{
    source_mapped::{SourceMapped, SourceRange},
    source_mapper::SourceId,
};

pub struct Tokenizer<'a> {
    source: Option<SourceId>,
    chars: Peekable<CharIndices<'a>>,
    curr_pos: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    Number,
    Boolean(bool),
    Identifier,
    Dot,
    Apostrophe,
    String,
    Undefined,
}

pub type Token = SourceMapped<TokenType>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenizeErrorType {
    UnexpectedCharacter,
    UnterminatedString,
    UnsupportedEscapeSequence,
}

pub type TokenizeError = SourceMapped<TokenizeErrorType>;

impl<'a> Tokenizer<'a> {
    pub fn new<T: AsRef<str>>(string: &'a T, source: Option<SourceId>) -> Self {
        Tokenizer {
            source,
            chars: string.as_ref().char_indices().peekable(),
            curr_pos: 0,
        }
    }

    pub fn curr_pos_as_source_range(&self) -> SourceRange {
        (self.curr_pos, self.curr_pos, self.source)
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

    fn chomp(&mut self) {
        self.accept(|_char| true);
    }

    fn peek<F: Fn(char) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(&(_pos, next_char)) = self.chars.peek() {
            if predicate(next_char) {
                true
            } else {
                false
            }
        } else {
            false
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

    fn try_accept_sharp(&mut self) -> Option<Result<TokenType, TokenizeErrorType>> {
        if self.accept_char('#') {
            let mut chars = vec![];
            loop {
                if let Some(&(pos, next_char)) = self.chars.peek() {
                    if is_ident_char(next_char) {
                        self.chars.next();
                        self.curr_pos = pos + next_char.len_utf8();
                        chars.push(next_char);
                        continue;
                    }
                }
                break;
            }
            let value: String = chars.into_iter().collect();
            let token = match value.as_str() {
                "t" => TokenType::Boolean(true),
                "f" => TokenType::Boolean(false),

                // This isn't documented in R5RS, but it's how try.scheme.org works...
                "!void" => TokenType::Undefined,

                _ => return Some(Err(TokenizeErrorType::UnexpectedCharacter)),
            };
            Some(Ok(token))
        } else {
            None
        }
    }

    fn try_accept_string(&mut self) -> Option<Result<TokenType, TokenizeErrorType>> {
        if self.accept_char('"') {
            loop {
                if self.accept_char('\\') {
                    if !self.accept(|c| matches!(c, '\\' | '"' | 'n')) {
                        return Some(Err(TokenizeErrorType::UnsupportedEscapeSequence));
                    }
                } else if self.accept_char('"') {
                    return Some(Ok(TokenType::String));
                } else if self.is_at_end() {
                    return Some(Err(TokenizeErrorType::UnterminatedString));
                } else {
                    self.chomp();
                }
            }
        } else {
            None
        }
    }

    fn try_accept_number(&mut self) -> Option<Result<TokenType, TokenizeErrorType>> {
        let mut found_decimals = 0;
        let mut found_digit = false;
        let start_pos = self.curr_pos;
        let found_plus_or_minus = self.accept(|char| char == '+' || char == '-');
        loop {
            if self.accept_char('.') {
                found_decimals += 1;
            } else if self.accept(|char| char.is_numeric()) {
                found_digit = true;
            } else {
                break;
            }
        }
        if found_digit && found_decimals <= 1 {
            Some(Ok(TokenType::Number))
        } else if found_decimals == 1 && !found_plus_or_minus && !self.peek(is_ident_char) {
            Some(Ok(TokenType::Dot))
        } else if self.curr_pos > start_pos {
            self.chomp_while(is_ident_char);
            Some(Ok(TokenType::Identifier))
        } else {
            None
        }
    }

    fn accept_identifier(&mut self) -> bool {
        if !self.accept(|char: char| !char.is_numeric() && is_ident_char(char)) {
            return false;
        }
        self.chomp_while(is_ident_char);
        true
    }

    fn chomp_whitespace(&mut self) {
        self.chomp_while(|char| char.is_whitespace());
    }

    fn accept_comment(&mut self) -> bool {
        if self.accept_char(';') {
            self.chomp_while(|char| char != '\n');
            true
        } else {
            false
        }
    }
}

fn is_ident_char(char: char) -> bool {
    !char.is_whitespace()
        && char != '('
        && char != ')'
        && char != ';'
        && char != '#'
        && char != '\''
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.chomp_whitespace();
            if !self.accept_comment() {
                break;
            }
        }
        if self.is_at_end() {
            return None;
        }
        let token_start = self.curr_pos;
        let token: Result<TokenType, TokenizeErrorType> = if self.accept_char('(') {
            Ok(TokenType::LeftParen)
        } else if self.accept_char(')') {
            Ok(TokenType::RightParen)
        } else if self.accept_char('\'') {
            Ok(TokenType::Apostrophe)
        } else if let Some(result) = self.try_accept_string() {
            result
        } else if let Some(result) = self.try_accept_number() {
            result
        } else if let Some(result) = self.try_accept_sharp() {
            result
        } else if self.accept_identifier() {
            Ok(TokenType::Identifier)
        } else {
            Err(TokenizeErrorType::UnexpectedCharacter)
        };
        let source = (token_start, self.curr_pos, self.source);
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
        let tokenizer = Tokenizer::new(&string, None);
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
                (Ok(Identifier), "..5"),
            ],
        )
    }

    #[test]
    fn dot_works() {
        test_tokenize(".", &[(Ok(Dot), ".")]);
        test_tokenize(". 32", &[(Ok(Dot), "."), (Ok(Number), "32")]);
        test_tokenize("1. .", &[(Ok(Number), "1."), (Ok(Dot), ".")]);
    }

    #[test]
    fn identifiers_starting_with_periods_work() {
        test_tokenize("..", &[(Ok(Identifier), "..")]);
        test_tokenize("..5a+(", &[(Ok(Identifier), "..5a+"), (Ok(LeftParen), "(")]);
        test_tokenize(".. 32", &[(Ok(Identifier), ".."), (Ok(Number), "32")]);
        test_tokenize("1. ...", &[(Ok(Number), "1."), (Ok(Identifier), "...")]);
    }

    #[test]
    fn plus_and_minus_work() {
        test_tokenize(
            "+3 -4 + 3 - 4",
            &[
                (Ok(Number), "+3"),
                (Ok(Number), "-4"),
                (Ok(Identifier), "+"),
                (Ok(Number), "3"),
                (Ok(Identifier), "-"),
                (Ok(Number), "4"),
            ],
        );
    }

    #[test]
    fn identifier_works() {
        test_tokenize(
            "hi there? ",
            &[(Ok(Identifier), "hi"), (Ok(Identifier), "there?")],
        )
    }

    #[test]
    fn booleans_work() {
        test_tokenize(
            " #t  #f ",
            &[(Ok(Boolean(true)), "#t"), (Ok(Boolean(false)), "#f")],
        )
    }

    #[test]
    fn comment_works() {
        test_tokenize(
            "hi ; here is a comment\n there ",
            &[(Ok(Identifier), "hi"), (Ok(Identifier), "there")],
        )
    }

    #[test]
    fn string_works() {
        test_tokenize(r#"  "hello"  "#, &[(Ok(String), r#""hello""#)]);
        test_tokenize(r#"  "hi \n bub"  "#, &[(Ok(String), r#""hi \n bub""#)]);
        test_tokenize(r#"  "hi \" bub"  "#, &[(Ok(String), r#""hi \" bub""#)]);
        test_tokenize(r#"  "hi \\ bub"  "#, &[(Ok(String), r#""hi \\ bub""#)]);
        test_tokenize(
            r#"  "hi \"#,
            &[(
                Err(TokenizeErrorType::UnsupportedEscapeSequence),
                r#""hi \"#,
            )],
        );
        test_tokenize(
            r#"  "hi "#,
            &[(Err(TokenizeErrorType::UnterminatedString), r#""hi "#)],
        );
    }
}

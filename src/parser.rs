use std::rc::Rc;

use crate::{
    source_mapped::{SourceMappable, SourceMapped},
    source_mapper::SourceId,
    string_interner::{InternedString, StringInterner},
    tokenizer::{Token, TokenType, TokenizeError, TokenizeErrorType, Tokenizer},
};

#[derive(Debug)]
pub enum ParseErrorType {
    Tokenize(TokenizeErrorType),
    InvalidNumber,
    MissingRightParen,
    UnexpectedRightParen,
}

pub type ParseError = SourceMapped<ParseErrorType>;

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        ParseErrorType::Tokenize(value.0).source_mapped(value.1)
    }
}

#[derive(Debug)]
pub enum ExpressionValue {
    Number(f64),
    Symbol(InternedString),
    Boolean(bool),
    Combination(Rc<Vec<Expression>>),
}

pub type Expression = SourceMapped<ExpressionValue>;

pub struct Parser<'a> {
    string: &'a str,
    tokenizer: Tokenizer<'a>,
    interner: &'a mut StringInterner,
}

impl<'a> Parser<'a> {
    pub fn new(
        string: &'a str,
        tokenizer: Tokenizer<'a>,
        interner: &'a mut StringInterner,
    ) -> Self {
        Parser {
            string,
            tokenizer,
            interner,
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_token(&mut self, token: Token) -> Result<Expression, ParseError> {
        match token.0 {
            TokenType::LeftParen => {
                let mut expressions = vec![];
                loop {
                    match self.tokenizer.next() {
                        Some(Ok(nested_token)) => {
                            if nested_token.0 == TokenType::RightParen {
                                return Ok(ExpressionValue::Combination(Rc::new(expressions))
                                    .source_mapped(token.extend_range(&nested_token)));
                            } else {
                                expressions.push(self.parse_token(nested_token)?);
                            }
                        }
                        Some(Err(tokenize_error)) => return Err(tokenize_error.into()),
                        None => {
                            return Err(ParseErrorType::MissingRightParen.source_mapped(token.1));
                        }
                    }
                }
            }
            TokenType::RightParen => {
                Err(ParseErrorType::UnexpectedRightParen.source_mapped(token.1))
            }
            TokenType::Boolean(boolean) => {
                Ok(ExpressionValue::Boolean(boolean).source_mapped(token.1))
            }
            TokenType::Number => match token.source(&self.string).parse::<f64>() {
                Ok(number) => Ok(ExpressionValue::Number(number).source_mapped(token.1)),
                Err(_) => Err(ParseErrorType::InvalidNumber.source_mapped(token.1)),
            },
            TokenType::Identifier => {
                let string = self.interner.intern(token.source(&self.string));
                Ok(ExpressionValue::Symbol(string).source_mapped(token.1))
            }
        }
    }

    pub fn parse_all(self) -> Result<Vec<Expression>, ParseError> {
        self.into_iter().collect()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Expression, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.next() {
            Some(Ok(token)) => Some(self.parse_token(token)),
            Some(Err(tokenize_error)) => Some(Err(tokenize_error.into())),
            None => None,
        }
    }
}

pub fn parse(
    code: &str,
    interner: &mut StringInterner,
    source: Option<SourceId>,
) -> Result<Vec<Expression>, ParseError> {
    let parser = Parser::new(code, Tokenizer::new(&code, source), interner);
    parser.parse_all()
}

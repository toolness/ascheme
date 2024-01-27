use crate::{
    source_mapped::SourceMapped,
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

type ParseError = SourceMapped<ParseErrorType>;

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        SourceMapped(ParseErrorType::Tokenize(value.0), value.1)
    }
}

#[derive(Debug)]
pub enum ExpressionValue {
    Number(f64),
    Symbol(InternedString),
    Combination(Box<Vec<Expression>>),
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
                                let expression = ExpressionValue::Combination(Box::new(expressions));
                                let left_paren_begin = token.1.0;
                                let right_paren_end = nested_token.1.1;
                                return Ok(SourceMapped(
                                    expression,
                                    (left_paren_begin, right_paren_end),
                                ));
                            } else {
                                let expression = self.parse_token(nested_token)?;
                                expressions.push(expression);
                            }
                        }
                        Some(Err(tokenize_error)) => return Err(tokenize_error.into()),
                        None => {
                            return Err(SourceMapped(ParseErrorType::MissingRightParen, token.1));
                        }
                    }
                }
            }
            TokenType::RightParen => {
                Err(SourceMapped(ParseErrorType::UnexpectedRightParen, token.1))
            }
            TokenType::Number => match token.source(&self.string).parse::<f64>() {
                Ok(number) => Ok(SourceMapped(ExpressionValue::Number(number), token.1)),
                Err(_) => Err(SourceMapped(ParseErrorType::InvalidNumber, token.1)),
            },
            TokenType::Identifier => {
                let string = self.interner.intern(token.source(&self.string));
                Ok(SourceMapped(ExpressionValue::Symbol(string), token.1))
            }
        }
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

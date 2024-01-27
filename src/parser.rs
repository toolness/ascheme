use crate::{
    string_interner::{InternedString, StringInterner},
    tokenizer::{Token, TokenizeError, Tokenizer},
};

#[derive(Debug)]
pub enum ParseError {
    Tokenize(TokenizeError),
    InvalidNumber,
    MissingRightParen,
    UnexpectedRightParen,
}

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        ParseError::Tokenize(value)
    }
}

#[derive(Debug)]
pub enum Expression {
    Number(f64),
    Symbol(InternedString),
    Combination(Box<Vec<Expression>>),
}

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
    fn parse_token(
        &mut self,
        token: Token,
        start: usize,
        end: usize,
    ) -> Result<Expression, ParseError> {
        match token {
            Token::LeftParen => {
                let mut expressions = vec![];
                loop {
                    match self.tokenizer.next() {
                        Some((Ok(token), (start, end))) => {
                            if token == Token::RightParen {
                                return Ok(Expression::Combination(Box::new(expressions)));
                            } else {
                                let expression = self.parse_token(token, start, end)?;
                                expressions.push(expression);
                            }
                        }
                        Some((Err(tokenize_error), _range)) => return Err(tokenize_error.into()),
                        None => {
                            return Err(ParseError::MissingRightParen);
                        }
                    }
                }
            }
            Token::RightParen => Err(ParseError::UnexpectedRightParen),
            Token::Number => match &self.string[start..end].parse::<f64>() {
                Ok(number) => Ok(Expression::Number(*number)),
                Err(_) => Err(ParseError::InvalidNumber),
            },
            Token::Identifier => {
                let string = self.interner.intern(&self.string[start..end]);
                Ok(Expression::Symbol(string))
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Expression, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.next() {
            Some((Ok(token), (start, end))) => Some(self.parse_token(token, start, end)),
            Some((Err(tokenize_error), _range)) => Some(Err(tokenize_error.into())),
            None => None,
        }
    }
}

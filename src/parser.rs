use crate::{
    mutable_string::MutableString,
    pair::PairManager,
    source_mapped::{SourceMappable, SourceMapped},
    source_mapper::SourceId,
    string_interner::StringInterner,
    tokenizer::{Token, TokenType, TokenizeError, TokenizeErrorType, Tokenizer},
    value::{SourceValue, Value},
};

#[derive(Debug)]
pub enum ParseErrorType {
    Tokenize(TokenizeErrorType),
    InvalidNumber,
    MissingRightParen,
    UnexpectedEndOfFile,
    Expected(TokenType),
    Unexpected(TokenType),
}

pub type ParseError = SourceMapped<ParseErrorType>;

impl From<TokenizeError> for ParseError {
    fn from(value: TokenizeError) -> Self {
        ParseErrorType::Tokenize(value.0).source_mapped(value.1)
    }
}

pub struct Parser<'a> {
    string: &'a str,
    tokenizer: Tokenizer<'a>,
    interner: &'a mut StringInterner,
    pair_manager: &'a mut PairManager,
}

impl<'a> Parser<'a> {
    pub fn new(
        string: &'a str,
        tokenizer: Tokenizer<'a>,
        interner: &'a mut StringInterner,
        pair_manager: &'a mut PairManager,
    ) -> Self {
        Parser {
            string,
            tokenizer,
            interner,
            pair_manager,
        }
    }
}

impl<'a> Parser<'a> {
    fn expect_token(&mut self) -> Result<Token, ParseError> {
        match self.tokenizer.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(tokenize_error)) => Err(tokenize_error.into()),
            None => Err(ParseErrorType::UnexpectedEndOfFile
                .source_mapped(self.tokenizer.curr_pos_as_source_range())),
        }
    }

    fn expect_expression(&mut self) -> Result<SourceValue, ParseError> {
        let token = self.expect_token()?;
        self.parse_token(token)
    }

    fn expect_token_type(&mut self, token_type: TokenType) -> Result<Token, ParseError> {
        let token = self.expect_token()?;
        if token.0 != token_type {
            return Err(ParseErrorType::Expected(token_type).source_mapped(token.1));
        }
        Ok(token)
    }

    fn parse_token(&mut self, token: Token) -> Result<SourceValue, ParseError> {
        match token.0 {
            TokenType::LeftParen => {
                let mut expressions = vec![];
                loop {
                    match self.tokenizer.next() {
                        Some(Ok(nested_token)) => {
                            if nested_token.0 == TokenType::RightParen {
                                return Ok(self
                                    .pair_manager
                                    .vec_to_list(expressions)
                                    .source_mapped(token.extend_range(&nested_token.1)));
                            } else if nested_token.0 == TokenType::Dot {
                                if expressions.is_empty() {
                                    return Err(ParseErrorType::Unexpected(TokenType::Dot)
                                        .source_mapped(nested_token.1));
                                }
                                let final_value = self.expect_expression()?;
                                let right_paren = self.expect_token_type(TokenType::RightParen)?;
                                return Ok(self
                                    .pair_manager
                                    .vec_to_pair(expressions, final_value)
                                    .source_mapped(right_paren.1));
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
                Err(ParseErrorType::Unexpected(TokenType::RightParen).source_mapped(token.1))
            }
            TokenType::Apostrophe => {
                let quoted_expression = self.expect_expression()?;
                let end_range = quoted_expression.1;
                let expressions = vec![
                    Value::Symbol(self.interner.intern("quote")).source_mapped(token.1),
                    quoted_expression,
                ];
                Ok(self
                    .pair_manager
                    .vec_to_list(expressions)
                    .source_mapped(token.extend_range(&end_range)))
            }
            TokenType::Dot => {
                Err(ParseErrorType::Unexpected(TokenType::Dot).source_mapped(token.1))
            }
            TokenType::Boolean(boolean) => Ok(Value::Boolean(boolean).source_mapped(token.1)),
            TokenType::Number => match token.source(&self.string).parse::<f64>() {
                Ok(number) => Ok(Value::Number(number).source_mapped(token.1)),
                Err(_) => Err(ParseErrorType::InvalidNumber.source_mapped(token.1)),
            },
            TokenType::String => {
                Ok(Value::String(self.parse_string(token.source(&self.string)))
                    .source_mapped(token.1))
            }
            TokenType::Identifier => {
                let string = self.interner.intern(token.source(&self.string));
                Ok(Value::Symbol(string).source_mapped(token.1))
            }
        }
    }

    fn parse_string(&self, repr: &str) -> MutableString {
        let mut chars: Vec<char> = Vec::with_capacity(repr.len());
        // The `skip(1)` skips the opening quote.
        let mut is_escaped = false;
        for char in repr.chars().skip(1) {
            if is_escaped {
                if char == 'n' {
                    chars.push('\n');
                } else {
                    chars.push(char);
                }
                is_escaped = false;
            } else {
                if char == '\\' {
                    is_escaped = true;
                } else {
                    chars.push(char);
                }
            }
        }
        chars.pop(); // Remove closing quote.
        let string: String = chars.into_iter().collect();
        MutableString::new(string)
    }

    pub fn parse_all(self) -> Result<Vec<SourceValue>, ParseError> {
        self.into_iter().collect()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<SourceValue, ParseError>;

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
    pair_manager: &mut PairManager,
    source: Option<SourceId>,
) -> Result<Vec<SourceValue>, ParseError> {
    let parser = Parser::new(code, Tokenizer::new(&code, source), interner, pair_manager);
    parser.parse_all()
}

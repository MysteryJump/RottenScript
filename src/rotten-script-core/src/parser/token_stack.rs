use crate::lexer::{reserved_word::ReservedWord, token::Token};

use super::parse_error::ParseError;

pub struct TokenStack<'a> {
    tokens: &'a [Token],
    index: i32,
    current: Option<Token>,
}

impl<'a> Iterator for TokenStack<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next() {
            self.index += 1;
            self.current = Some(self.tokens[self.index as usize].clone());
            self.current.clone()
        } else {
            None
        }
    }
}

impl<'a> TokenStack<'a> {
    pub fn new(tokens: &'a [Token]) -> TokenStack<'a> {
        TokenStack {
            tokens,
            index: -1,
            current: None,
        }
    }

    pub fn has_next(&self) -> bool {
        self.tokens.len() as i32 != self.index + 1
    }

    pub fn look_ahead(&self, range: i32) -> Option<Token> {
        if self.index + range < self.tokens.len() as i32 {
            Some(self.tokens[(self.index + range) as usize].clone())
        } else {
            None
        }
    }

    pub fn ind(&self) -> i32 {
        self.index
    }
    pub fn peek(&self) -> Option<Token> {
        self.current.clone()
    }

    pub fn consume_reserved(&mut self, reserved: ReservedWord) -> Result<(), ParseError> {
        self.scan_reserved(reserved)?;
        self.next();
        Ok(())
    }

    pub fn scan_reserved(&mut self, reserved: ReservedWord) -> Result<(), ParseError> {
        let unexpected_token = "Unexpected token";
        let unexpected_eof = "unexpected eof";
        if let Some(next) = self.look_ahead(1) {
            match next {
                Token::Reserved(r) => {
                    if r != reserved {
                        Err(ParseError::new(unexpected_token))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(ParseError::new(unexpected_token)),
            }
        } else {
            Err(ParseError::new(unexpected_eof))
        }
    }
}

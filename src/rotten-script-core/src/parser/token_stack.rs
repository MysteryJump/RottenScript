use crate::lexer::{
    reserved_word::ReservedWord,
    token::{Token, TokenBase},
};

use super::parse_error::ParseError;
// FIXME: None also indicates lexing error, this will fix use Token instead of TokenBase.
pub struct TokenStack<'a> {
    tokens: &'a [Token],
    index: i32,
    current: Option<Token>,
}

impl<'a> Iterator for TokenStack<'a> {
    type Item = TokenBase;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next() {
            self.index += 1;
            self.current = Some(self.tokens[self.index as usize].clone());
            self.current.as_ref().unwrap().get_token().clone()
        } else {
            None
        }
    }
}

impl<'a> TokenStack<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        TokenStack {
            tokens,
            index: -1,
            current: None,
        }
    }

    pub fn has_next(&self) -> bool {
        self.tokens.len() as i32 != self.index + 1
    }

    pub fn look_ahead(&self, range: i32) -> Option<TokenBase> {
        if self.index + range < self.tokens.len() as i32 {
            self.tokens[(self.index + range) as usize]
                .get_token()
                .clone()
        } else {
            None
        }
    }

    pub fn ind(&self) -> i32 {
        self.index
    }
    pub fn peek(&self) -> Option<TokenBase> {
        match &self.current {
            Some(tb) => tb.get_token().clone(),
            None => None,
        }
    }

    pub fn consume_reserved(&mut self, reserved: ReservedWord) -> Result<(), ParseError> {
        self.scan_reserved(reserved)?;
        self.next();
        Ok(())
    }

    pub fn scan_reserved(&self, reserved: ReservedWord) -> Result<(), ParseError> {
        let unexpected_token = "Unexpected token";
        let unexpected_eof = "unexpected eof";
        if let Some(next) = self.look_ahead(1) {
            match next {
                TokenBase::Reserved(r) => {
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
        // if let Some(next) = self.look_ahead(1) {
        //     match next.get_token() {
        //         Some(TokenBase::Reserved(r)) => {
        //             if r != &reserved {
        //                 Err(ParseError::new(unexpected_token))
        //             } else {
        //                 Ok(())
        //             }
        //         }
        //         _ => Err(ParseError::new(unexpected_token)),
        //     }
        // } else {
        //     Err(ParseError::new(unexpected_eof))
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_utilities() {
        let tokens = [
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Arrow)), 0, 0, 0),
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Comma)), 0, 0, 0),
            Token::new(Ok(TokenBase::Identifier("test".to_string())), 0, 0, 0),
        ];
        let mut token_stack = TokenStack::new(&tokens);
        assert_eq!(true, token_stack.has_next());
        assert_eq!(None, token_stack.current);
        assert_eq!(
            Some(tokens[0].get_token().as_ref().unwrap().clone()),
            token_stack.next()
        );
        assert_eq!(0, token_stack.ind());
        assert_eq!(
            Some(tokens[1].get_token().as_ref().unwrap().clone()),
            token_stack.next()
        );
        assert_eq!(
            Some(tokens[2].get_token().as_ref().unwrap().clone()),
            token_stack.look_ahead(1)
        );
        assert_eq!(
            Some(tokens[1].get_token().as_ref().unwrap().clone()),
            token_stack.peek()
        );
        assert_eq!(
            Some(tokens[2].get_token().as_ref().unwrap().clone()),
            token_stack.next()
        );
        assert_eq!(false, token_stack.has_next());
    }

    #[test]
    fn test_scan_reserved() {
        let tokens = [
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Arrow)), 0, 0, 0),
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Comma)), 0, 0, 0),
            Token::new(Ok(TokenBase::Identifier("test".to_string())), 0, 0, 0),
        ];
        let mut token_stack = TokenStack::new(&tokens);
        token_stack.scan_reserved(ReservedWord::Arrow).unwrap();
        token_stack.next();
        token_stack.scan_reserved(ReservedWord::Comma).unwrap();
        token_stack.next();
        if token_stack.scan_reserved(ReservedWord::Arrow).is_ok() {
            panic!()
        }
    }

    #[test]
    fn test_consume_reserved() {
        let tokens = [
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Arrow)), 0, 0, 0),
            Token::new(Ok(TokenBase::Reserved(ReservedWord::Comma)), 0, 0, 0),
            Token::new(Ok(TokenBase::Identifier("test".to_string())), 0, 0, 0),
        ];
        let mut token_stack = TokenStack::new(&tokens);
        token_stack.consume_reserved(ReservedWord::Arrow).unwrap();
        token_stack.consume_reserved(ReservedWord::Comma).unwrap();
        if token_stack.consume_reserved(ReservedWord::Arrow).is_ok() {
            panic!()
        }
    }
}

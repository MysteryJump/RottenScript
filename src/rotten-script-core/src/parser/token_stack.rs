use crate::lexer::{
    reserved_word::ReservedWord,
    token::{Token, TokenBase},
};

use super::{
    invalid_syntax::{ExpectedActuallyTokenPair, InvalidSyntax, InvalidSyntaxType},
    parse_error::ParseError,
};
// FIXME: None also indicates lexing error, this will fix use Token instead of TokenBase.
pub struct TokenStack<'a> {
    tokens: &'a [Token],
    index: i32,
    current: Option<Token>,
}

impl<'a> TokenStack<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<TokenBase> {
        if self.has_next() {
            self.index += 1;
            self.current = Some(self.tokens[self.index as usize].clone());
            self.current.as_ref().unwrap().get_token().clone()
        } else {
            None
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
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

    pub fn nth(&self, range: usize) -> Option<Token> {
        if (self.index + range as i32) < self.tokens.len() as i32 {
            Some(self.tokens[(self.index + range as i32) as usize].clone())
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

    pub fn peek_token(&self) -> Option<Token> {
        self.current.clone()
    }

    pub fn consume_reserved(&mut self, reserved: ReservedWord) -> Result<(), InvalidSyntax> {
        self.scan_reserved(reserved)?;
        self.next();
        Ok(())
    }

    pub fn scan_reserved(&self, reserved: ReservedWord) -> Result<(), InvalidSyntax> {
        if let Some(next) = self.nth(1) {
            match next.get_token() {
                Some(TokenBase::Reserved(r)) => {
                    if r != &reserved {
                        Err(InvalidSyntax::new(
                            next.get_token_position(),
                            InvalidSyntaxType::ExpectedNext(ExpectedActuallyTokenPair(
                                vec![TokenBase::Reserved(reserved)],
                                next,
                            )),
                        ))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(InvalidSyntax::new(
                    next.get_token_position(),
                    InvalidSyntaxType::ExpectedNext(ExpectedActuallyTokenPair(
                        vec![TokenBase::Reserved(reserved)],
                        next,
                    )),
                )),
            }
        } else {
            let tk = self.peek_token().unwrap();
            Err(InvalidSyntax::new(
                tk.get_token_position(),
                InvalidSyntaxType::UnexpectedEof,
            ))
        }
    }

    pub fn skip_reserved_until(&mut self, _reserved: ReservedWord) -> Result<(), InvalidSyntax> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    #[test]
    fn test_basic_utilities() {
        let tokens = [
            create_token_data(TokenBase::Reserved(ReservedWord::Arrow)),
            create_token_data(TokenBase::Reserved(ReservedWord::Comma)),
            create_token_data(TokenBase::Identifier("test".to_string())),
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
            create_token_data(TokenBase::Reserved(ReservedWord::Arrow)),
            create_token_data(TokenBase::Reserved(ReservedWord::Comma)),
            create_token_data(TokenBase::Identifier("test".to_string())),
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
            create_token_data(TokenBase::Reserved(ReservedWord::Arrow)),
            create_token_data(TokenBase::Reserved(ReservedWord::Comma)),
            create_token_data(TokenBase::Identifier("test".to_string())),
        ];

        let mut token_stack = TokenStack::new(&tokens);
        token_stack.consume_reserved(ReservedWord::Arrow).unwrap();
        token_stack.consume_reserved(ReservedWord::Comma).unwrap();
        if token_stack.consume_reserved(ReservedWord::Arrow).is_ok() {
            panic!()
        }
    }

    fn create_token_data(token_base: TokenBase) -> Token {
        Token::new(Ok(token_base), 0, 0, 0, Rc::new("".to_string()))
    }
}

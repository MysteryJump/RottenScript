use super::{ast::Ast, non_terminal::NonTerminal, parse_error::ParseError, Parser};
use crate::lexer::{reserved_word::ReservedWord, token::TokenBase};

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<Ast, ParseError> {
        Ok(Ast::new_node_with_leaves(
            NonTerminal::Expression,
            vec![match self.tokens.look_ahead(1).unwrap() {
                TokenBase::Identifier(_) => self.parse_call_expression()?,
                TokenBase::String(_)
                | TokenBase::Number(_)
                | TokenBase::Reserved(ReservedWord::True)
                | TokenBase::Reserved(ReservedWord::False) => {
                    Ast::new_leaf(self.tokens.next_token().unwrap())
                }
                TokenBase::Reserved(ReservedWord::LeftParenthesis) => {
                    // NOTE: after read all of item of inner parentheses, if there is =>, it is arrow expression
                    self.parse_function_expression()?
                }
                _ => panic!(),
            }],
        ))
    }

    fn parse_call_expression(&mut self) -> Result<Ast, ParseError> {
        let mut idents = vec![Ast::new_leaf(self.tokens.next_token().unwrap())];

        while let Some(TokenBase::Reserved(ReservedWord::Period)) = self.tokens.look_ahead(1) {
            let ne = self.tokens.next_token();
            if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
                idents.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
            } else {
                let tk = self.tokens.next_token();
                self.handle_expected_actually_error(
                    tk,
                    vec![TokenBase::default_identifier()],
                    ne.unwrap(),
                );
            }
        }

        let args = self.parse_args()?;
        idents.push(args);
        Ok(Ast::new_node_with_leaves(
            NonTerminal::CallExpression,
            idents,
        ))
    }
}

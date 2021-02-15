use crate::lexer::{reserved_word::ReservedWord, token::TokenBase};

use super::{
    ast::Ast, non_terminal::NonTerminal, parse_error::ParseError, InvalidSyntaxResultHandler,
    Parser,
};

impl<'a> Parser<'a> {
    fn parse_named_import_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        // self.tokens.consume_reserved(ReservedWord::LeftCurly)?;
        self.tokens
            .consume_reserved2(ReservedWord::LeftCurly)
            .handle_consume(self);
        // called2
        let mut asts = Vec::new();
        loop {
            if let Some(s) = self.tokens.look_ahead(1) {
                match s {
                    TokenBase::Identifier(_) => {
                        let next_token = self.tokens.next_token().unwrap();
                        asts.push(Ast::new_leaf(next_token));
                        match self.tokens.look_ahead(1) {
                            Some(TokenBase::Reserved(r))
                                if r == ReservedWord::Comma || r == ReservedWord::RightCurly =>
                            {
                                match r {
                                    ReservedWord::Comma => {
                                        self.tokens.next();
                                    }
                                    ReservedWord::RightCurly => {
                                        self.tokens.next();
                                        break;
                                    }
                                    _ => panic!(),
                                }
                            }
                            Some(_) | None => {
                                self.handle_expected_actually_error(
                                    self.tokens.nth(1),
                                    vec![
                                        TokenBase::Reserved(ReservedWord::Comma),
                                        TokenBase::Reserved(ReservedWord::RightCurly),
                                    ],
                                    self.tokens.peek_token().unwrap(),
                                );
                                // called
                                return Err(ParseError::new("unexpected eof or token"));
                            }
                        }
                    }
                    _ => {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![TokenBase::default_identifier()],
                            self.tokens.peek_token().unwrap(),
                        );
                        // called
                        return Err(ParseError::new("unexpected token"));
                    }
                }
            } else {
                self.handle_unexpected_eof_error(self.tokens.peek_token().unwrap());
                // called
                return Err(ParseError::new("unexpected eof"));
            }
        }
        self.tokens
            .consume_reserved2(ReservedWord::From)
            .handle_consume(self);
        // self.tokens.consume_reserved(ReservedWord::From)?;
        // called2
        if let Some(TokenBase::String(_)) = self.tokens.look_ahead(1) {
            asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
            Ok(Ast::new_node_with_leaves(
                NonTerminal::NamedImportDeclaration,
                asts,
            ))
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::default_string()],
                self.tokens.peek_token().unwrap(),
            );
            // called
            Err(ParseError::new("unexpected token or eof"))
        }
    }

    // DefaultImportDeclaration = "import" , Identifier , "from" , (DoubleQuotesString | SingleQuotesString);
    fn parse_default_import_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let mut asts = Vec::new();
        if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
            asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::default_identifier()],
                self.tokens.peek_token().unwrap(),
            );
            // called
            return Err(ParseError::new("unexpected token or eof"));
        }
        self.tokens
            .consume_reserved2(ReservedWord::From)
            .handle_consume(self);
        // called
        // self.tokens.consume_reserved(ReservedWord::From)?;
        if let Some(TokenBase::String(_)) = self.tokens.look_ahead(1) {
            asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
            Ok(Ast::new_node_with_leaves(
                NonTerminal::DefaultImportDeclaration,
                asts,
            ))
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::default_string()],
                self.tokens.peek_token().unwrap(),
            );
            // called
            Err(ParseError::new("unexpected token or eof"))
        }
    }

    // ImportDeclaration = (NamedImportDeclaration | DefaultImportDeclaration) , ";";
    pub fn parse_import_declaration(&mut self) -> Result<Ast, ParseError> {
        let result = if let Some(next) = self.tokens.look_ahead(2) {
            match next {
                TokenBase::Reserved(ReservedWord::LeftCurly) => Ok(Ast::new_node_with_leaves(
                    NonTerminal::ImportDeclaration,
                    vec![self.parse_named_import_declaration()?],
                )),
                TokenBase::Identifier(_) => Ok(Ast::new_node_with_leaves(
                    NonTerminal::ImportDeclaration,
                    vec![self.parse_default_import_declaration()?],
                )),
                _ => {
                    self.handle_expected_actually_error(
                        self.tokens.nth(2),
                        vec![
                            TokenBase::Reserved(ReservedWord::LeftCurly),
                            TokenBase::default_identifier(),
                        ],
                        self.tokens.nth(1).unwrap(),
                    );
                    // called
                    return Err(ParseError::new("unexpected token"));
                }
            }
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(2),
                vec![
                    TokenBase::Reserved(ReservedWord::LeftCurly),
                    TokenBase::default_identifier(),
                ],
                self.tokens.nth(1).unwrap(),
            );
            // called
            Err(ParseError::new("unexpected eof"))
        };
        self.tokens
            .consume_reserved2(ReservedWord::SemiColon)
            .handle_consume(self);
        // called
        // self.tokens.consume_reserved(ReservedWord::SemiColon)?;
        result
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::lexer::{reserved_word::ReservedWord::*, token::TokenBase::*};

    use super::{
        super::ast_type::AstType::*, super::non_terminal::NonTerminal::*,
        super::token_stack::TokenStack, Parser,
    };

    #[test]
    fn parse_named_import_declaration_test() {
        let use_state = Identifier("useState".to_string());
        let use_effect = Identifier("useEffect".to_string());
        let react = String("react".to_string());
        let tokens = vec![
            Reserved(Import),
            Reserved(LeftCurly),
            use_state.clone(),
            Reserved(Comma),
            use_effect.clone(),
            Reserved(RightCurly),
            Reserved(From),
            react.clone(),
        ];
        let token_list = to_token_list(&tokens);
        let mut token_stack = TokenStack::new(&token_list);
        let mut parser = Parser::new(&mut token_stack);
        let ast = parser.parse_named_import_declaration().unwrap();
        assert_eq!(NonTerminal(NamedImportDeclaration), ast.ast_type);
        let children = ast.children.as_ref().unwrap();
        assert_eq!(count_without_reserved_token_base(&tokens), children.len());
        assert_eq!(use_state, children[0].token.clone().unwrap());
        assert_eq!(use_effect, children[1].token.clone().unwrap());
        assert_eq!(react, children[2].token.clone().unwrap());
    }
    #[test]
    fn parse_default_import_declaration_test() {
        let react = String("react".to_string());
        let upper_react = Identifier("React".to_string());
        let tokens = vec![
            Reserved(Import),
            upper_react.clone(),
            Reserved(From),
            react.clone(),
        ];
        let token_list = to_token_list(&tokens);
        let mut token_stack = TokenStack::new(&token_list);
        let mut parser = Parser::new(&mut token_stack);
        let ast = parser.parse_default_import_declaration().unwrap();
        assert_eq!(NonTerminal(DefaultImportDeclaration), ast.ast_type);
        let children = ast.children.as_ref().unwrap();
        assert_eq!(count_without_reserved_token_base(&tokens), children.len());
        assert_eq!(upper_react, children[0].token.clone().unwrap());
        assert_eq!(react, children[1].token.clone().unwrap());
    }

    #[test]
    fn parse_import_declaration_test() {}

    fn count_without_reserved_token(tokens: &[crate::lexer::token::Token]) -> usize {
        tokens
            .iter()
            .filter(|x| !matches!(x.get_token().as_ref().unwrap(), Reserved(_)))
            .count()
    }

    fn count_without_reserved_token_base(tokens: &[crate::lexer::token::TokenBase]) -> usize {
        tokens.iter().filter(|x| !matches!(x, Reserved(_))).count()
    }
    fn to_token_list(
        token_bases: &[crate::lexer::token::TokenBase],
    ) -> Vec<crate::lexer::token::Token> {
        let fp = Rc::new("".to_string());

        token_bases
            .iter()
            .map(|x| crate::lexer::token::Token::new(Ok(x.clone()), 0, 0, 0, fp.clone()))
            .collect()
    }
}

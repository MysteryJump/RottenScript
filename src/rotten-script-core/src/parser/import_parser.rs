use crate::lexer::{reserved_word::ReservedWord, token::Token};

use super::{ast::Ast, non_terminal::NonTerminal, parse_error::ParseError, Parser};

impl<'a> Parser<'a> {
    fn parse_named_import_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        self.tokens.consume_reserved(ReservedWord::LeftCurly)?;
        let mut asts = Vec::new();
        loop {
            if let Some(s) = self.tokens.look_ahead(1) {
                match s {
                    Token::Identifier(_) => {
                        asts.push(Ast::new_leaf(self.tokens.next().unwrap()));
                        match self.tokens.look_ahead(1) {
                            Some(Token::Reserved(r)) => match r {
                                ReservedWord::Comma => {
                                    self.tokens.next();
                                }
                                ReservedWord::RightCurly => {
                                    self.tokens.next();
                                    break;
                                }
                                _ => return Err(ParseError::new("unexpected token")),
                            },
                            Some(_) | None => {
                                return Err(ParseError::new("unexpected eof or token"))
                            }
                        }
                    }
                    _ => return Err(ParseError::new("unexpected token")),
                }
            } else {
                return Err(ParseError::new("unexpected eof"));
            }
        }
        self.tokens.consume_reserved(ReservedWord::From)?;
        if let Some(Token::String(s)) = self.tokens.next() {
            asts.push(Ast::new_leaf(Token::String(s)));
            Ok(Ast::new_node_with_leaves(
                NonTerminal::NamedImportDeclaration,
                asts,
            ))
        } else {
            Err(ParseError::new("unexpected token or eof"))
        }
    }

    // DefaultImportDeclaration = "import" , Identifier , "from" , (DoubleQuotesString | SingleQuotesString);
    fn parse_default_import_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let mut asts = Vec::new();
        if let Some(Token::Identifier(ident)) = self.tokens.next() {
            asts.push(Ast::new_leaf(Token::Identifier(ident)));
        } else {
            return Err(ParseError::new("unexpected token or eof"));
        }
        self.tokens.consume_reserved(ReservedWord::From)?;
        if let Some(Token::String(s)) = self.tokens.next() {
            asts.push(Ast::new_leaf(Token::String(s)));
            Ok(Ast::new_node_with_leaves(
                NonTerminal::DefaultImportDeclaration,
                asts,
            ))
        } else {
            Err(ParseError::new("unexpected token or eof"))
        }
    }

    // ImportDeclaration = NamedImportDeclaration | DefaultImportDeclaration;
    pub fn parse_import_declaration(&mut self) -> Result<Ast, ParseError> {
        if let Some(next) = self.tokens.look_ahead(2) {
            match next {
                Token::Reserved(ReservedWord::LeftCurly) => Ok(Ast::new_node_with_leaves(
                    NonTerminal::ImportDeclaration,
                    vec![self.parse_named_import_declaration()?],
                )),
                Token::Identifier(_) => Ok(Ast::new_node_with_leaves(
                    NonTerminal::ImportDeclaration,
                    vec![self.parse_default_import_declaration()?],
                )),
                _ => Err(ParseError::new("unexpected token")),
            }
        } else {
            Err(ParseError::new("unexpected eof"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{reserved_word::ReservedWord::*, token::Token::*};

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
        let mut token_stack = TokenStack::new(&tokens);
        let mut parser = Parser::new(&mut token_stack, &logger);
        let ast = parser.parse_named_import_declaration().unwrap();
        assert_eq!(NonTerminal(NamedImportDeclaration), ast.ast_type);
        let children = ast.children.as_ref().unwrap();
        assert_eq!(count_without_reserved(&tokens), children.len());
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
        let mut token_stack = TokenStack::new(&tokens);
        let mut parser = Parser::new(&mut token_stack, &logger);
        let ast = parser.parse_default_import_declaration().unwrap();
        assert_eq!(NonTerminal(DefaultImportDeclaration), ast.ast_type);
        let children = ast.children.as_ref().unwrap();
        assert_eq!(count_without_reserved(&tokens), children.len());
        assert_eq!(upper_react, children[0].token.clone().unwrap());
        assert_eq!(react, children[1].token.clone().unwrap());
    }

    #[test]
    fn parse_import_declaration_test() {}

    fn count_without_reserved(tokens: &[crate::lexer::token::Token]) -> usize {
        tokens.iter().filter(|x| !matches!(x, Reserved(_))).count()
    }

    fn logger(_: &str) {}
}

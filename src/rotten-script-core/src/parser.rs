use std::vec;

use non_terminal::NonTerminal;
use parse_error::{ParseError, ParseError2};
use token_stack::TokenStack;

use crate::lexer::{
    reserved_word::ReservedWord,
    token::{Token, TokenBase},
};

use self::{
    ast::Ast,
    invalid_syntax::{ExpectedActuallyTokenPair, InvalidSyntax, InvalidSyntaxType},
};

pub mod ast;
pub mod ast_type;
mod invalid_syntax;
pub(crate) mod non_terminal;
mod parse_error;
pub mod token_stack;

mod import_parser;

pub struct Parser<'a> {
    pub tokens: &'a mut TokenStack<'a>,
    pub ast: Ast,
    parse_error: ParseError2,
}

trait InvalidSyntaxResultHandler {
    fn handle_scan(self, perror: &mut ParseError2);
    fn handle_consume(self, parser: &mut Parser);
}

impl InvalidSyntaxResultHandler for Result<(), InvalidSyntax> {
    fn handle_scan(self, perror: &mut ParseError2) {
        self.unwrap_or_else(|e| perror.add_error(e));
    }

    fn handle_consume(self, parser: &mut Parser) {
        self.unwrap_or_else(|e| {
            parser.parse_error.add_error(e);
            parser.tokens.next();
        })
    }
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut TokenStack<'a>) -> Parser<'a> {
        Parser {
            tokens,
            ast: Ast::new_node_with_leaves(NonTerminal::TranslationUnit, Vec::new()),
            parse_error: ParseError2::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        self.parse_translation_unit()
    }

    // TranslationUnit = { ImportDeclaration } , { { Attribute } , ExportableConstDeclaration };
    fn parse_translation_unit(&mut self) -> Result<(), ParseError> {
        while let Some(TokenBase::Reserved(ReservedWord::Import)) = self.tokens.look_ahead(1) {
            let import = self.parse_import_declaration()?;
            self.ast.add_child(import);
        }
        while self.tokens.has_next() {
            let c = match self.tokens.look_ahead(1) {
                Some(TokenBase::Reserved(ReservedWord::LeftSquareBracket)) => {
                    while Some(TokenBase::Reserved(ReservedWord::LeftSquareBracket))
                        == self.tokens.look_ahead(1)
                    {
                        let child = self.parse_attribute()?;
                        self.ast.add_child(child);
                    }
                    if Some(TokenBase::Reserved(ReservedWord::Const)) == self.tokens.look_ahead(1) {
                        self.parse_exportable_const_declaration()?
                    } else {
                        let target_token = self.tokens.nth(1);
                        self.handle_expected_actually_error(
                            target_token.clone(),
                            vec![TokenBase::Reserved(ReservedWord::Const)],
                            self.tokens.peek_token().unwrap(),
                        );
                        Ast::new_leaf(target_token.unwrap())
                    }
                }
                Some(TokenBase::Reserved(ReservedWord::Const))
                | Some(TokenBase::Reserved(ReservedWord::Export)) => {
                    self.parse_exportable_const_declaration()?
                }
                Some(_) | None => {
                    let target = self.tokens.nth(1);
                    self.handle_expected_actually_error(
                        self.tokens.nth(1),
                        vec![
                            TokenBase::Reserved(ReservedWord::LeftSquareBracket),
                            TokenBase::Reserved(ReservedWord::Const),
                            TokenBase::Reserved(ReservedWord::Export),
                        ],
                        self.tokens.peek_token().unwrap(),
                    );
                    Ast::new_leaf(target.unwrap())
                }
            };
            self.ast.add_child(c);
        }

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn parse_attribute(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let ast2 = match self.tokens.look_ahead(1) {
            Some(TokenBase::Identifier(_)) => {
                let attr_token = self.tokens.next_token().unwrap();
                self.tokens
                    .consume_reserved2(ReservedWord::RightSquareBracket)
                    .handle_consume(self);
                Ast::new_node_with_leaves(NonTerminal::Attribute, vec![Ast::new_leaf(attr_token)])
            }
            None | Some(_) => {
                let target_token = self.tokens.nth(1);
                self.handle_expected_actually_error(
                    target_token.clone(),
                    vec![TokenBase::default_identifier()],
                    self.tokens.peek_token().unwrap(),
                );
                Ast::new_leaf(target_token.unwrap())
            }
        };

        Ok(ast2)
    }

    // ExportableConstDeclaration = [ "export" , ["default"] ] , ConstDeclaration;
    fn parse_exportable_const_declaration(&mut self) -> Result<Ast, ParseError> {
        let mut ast = Vec::new();

        match self.tokens.look_ahead(1) {
            Some(TokenBase::Reserved(ReservedWord::Export)) => {
                ast.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                match self.tokens.look_ahead(1) {
                    Some(TokenBase::Reserved(ReservedWord::Default)) => {
                        ast.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                        self.tokens
                            .scan_reserved2(ReservedWord::Const)
                            .handle_scan(&mut self.parse_error);
                        ast.push(self.parse_const_declaration()?);
                    }
                    Some(TokenBase::Reserved(ReservedWord::Const)) => {
                        ast.push(self.parse_const_declaration()?);
                    }
                    Some(_) | None => {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![
                                TokenBase::Reserved(ReservedWord::Default),
                                TokenBase::Reserved(ReservedWord::Const),
                            ],
                            self.tokens.peek_token().unwrap(),
                        );
                    }
                }
            }
            Some(TokenBase::Reserved(ReservedWord::Const)) => {
                ast.push(self.parse_const_declaration()?);
            }
            Some(_) | None => {
                let target_token = self.tokens.nth(1);
                self.handle_expected_actually_error(
                    target_token,
                    vec![
                        TokenBase::Reserved(ReservedWord::Const),
                        TokenBase::Reserved(ReservedWord::Export),
                    ],
                    self.tokens.peek_token().unwrap(),
                );
            }
        }
        Ok(Ast::new_node_with_leaves(
            NonTerminal::ExportableConstDeclaration,
            ast,
        ))
    }

    fn parse_const_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        Ok(Ast::new_node_with_leaves(
            NonTerminal::ConstDeclaration,
            vec![self.parse_declaration_body()?],
        ))
    }

    #[allow(dead_code)]
    fn parse_let_declaration(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        Ok(Ast::new_node_with_leaves(
            NonTerminal::LetDeclaration,
            vec![self.parse_declaration_body()?],
        ))
    }

    fn parse_declaration_body(&mut self) -> Result<Ast, ParseError> {
        let ident_ast = if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
            Ast::new_leaf(self.tokens.next_token().unwrap())
        } else {
            let target_token = self.tokens.nth(1);
            self.handle_expected_actually_error(
                target_token.clone(),
                vec![TokenBase::default_identifier()],
                self.tokens.peek_token().unwrap(),
            );
            Ast::new_leaf(target_token.unwrap())
        };
        self.tokens
            .consume_reserved2(ReservedWord::Assign)
            .handle_consume(self);
        let expr_ast = self.parse_expression()?;
        if let Some(TokenBase::Reserved(ReservedWord::SemiColon)) = self.tokens.look_ahead(1) {
            self.tokens.next();
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::Reserved(ReservedWord::SemiColon)],
                self.tokens.peek_token().unwrap(),
            );
        }
        Ok(Ast::new_node_with_leaves(
            NonTerminal::DeclarationBody,
            vec![ident_ast, expr_ast],
        ))
    }

    fn parse_expression(&mut self) -> Result<Ast, ParseError> {
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

    fn parse_args(&mut self) -> Result<Ast, ParseError> {
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        let mut callers = Vec::new();

        if self.tokens.look_ahead(1) == Some(TokenBase::Reserved(ReservedWord::RightParenthesis)) {
            self.tokens.next();
            Ok(Ast::new_node_with_leaves(NonTerminal::Args, Vec::new()))
        } else {
            loop {
                match self.tokens.look_ahead(1) {
                    Some(TokenBase::String(_))
                    | Some(TokenBase::Number(_))
                    | Some(TokenBase::Identifier(_))
                    | Some(TokenBase::Reserved(ReservedWord::LeftParenthesis)) => {
                        callers.push(self.parse_expression()?);
                        match self.tokens.look_ahead(1) {
                            Some(TokenBase::Reserved(ReservedWord::Comma)) => {
                                self.tokens.next();
                            }
                            Some(TokenBase::Reserved(ReservedWord::RightParenthesis)) => {
                                self.tokens.next();
                                break;
                            }
                            Some(_) | None => {
                                self.handle_expected_actually_error(
                                    self.tokens.nth(1),
                                    vec![
                                        TokenBase::Reserved(ReservedWord::Comma),
                                        TokenBase::Reserved(ReservedWord::RightParenthesis),
                                    ],
                                    self.tokens.peek_token().unwrap(),
                                );
                            }
                        }
                    }
                    Some(_) | None => {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![
                                TokenBase::Reserved(ReservedWord::Comma),
                                TokenBase::Reserved(ReservedWord::RightParenthesis),
                            ],
                            self.tokens.peek_token().unwrap(),
                        );
                    }
                }
            }
            Ok(Ast::new_node_with_leaves(NonTerminal::Args, callers))
        }
    }

    fn parse_function_expression(&mut self) -> Result<Ast, ParseError> {
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        self.tokens
            .consume_reserved2(ReservedWord::Arrow)
            .handle_consume(self);
        Ok(Ast::new_node_with_leaves(
            NonTerminal::FunctionExpression,
            vec![self.parse_compound_expression()?],
        ))
    }

    fn parse_compound_expression(&mut self) -> Result<Ast, ParseError> {
        let mut expressions = Vec::new();
        self.tokens
            .consume_reserved2(ReservedWord::LeftCurly)
            .handle_consume(self);
        while let Some(tk) = self.tokens.look_ahead(1) {
            let exp = match tk {
                TokenBase::String(_) | TokenBase::Number(_) | TokenBase::Identifier(_) => {
                    self.parse_expression()?
                }
                TokenBase::Reserved(ReservedWord::LeftParenthesis) => self.parse_expression()?,
                _ => {
                    let target_token = self.tokens.nth(1);
                    self.handle_expected_actually_error(
                        target_token.clone(),
                        vec![
                            TokenBase::default_string(),
                            TokenBase::default_number(),
                            TokenBase::default_identifier(),
                            TokenBase::Reserved(ReservedWord::LeftParenthesis),
                        ],
                        self.tokens.peek_token().unwrap(),
                    );
                    Ast::new_leaf(target_token.unwrap())
                }
            };

            let has_semicolon = if let Some(TokenBase::Reserved(ReservedWord::SemiColon)) =
                self.tokens.look_ahead(1)
            {
                self.tokens.next();
                expressions.push(Ast::new_node_with_leaves(
                    NonTerminal::ExpressionStatement,
                    vec![exp],
                ));
                true
            } else {
                expressions.push(exp);
                false
            };

            if let Some(TokenBase::Reserved(ReservedWord::RightCurly)) = self.tokens.look_ahead(1) {
                break;
            } else if !has_semicolon {
                self.handle_expected_actually_error(
                    self.tokens.nth(1),
                    vec![TokenBase::Reserved(ReservedWord::SemiColon)],
                    self.tokens.peek_token().unwrap(),
                );
            }
        }
        self.tokens
            .consume_reserved2(ReservedWord::RightCurly)
            .handle_consume(self);

        Ok(Ast::new_node_with_leaves(
            NonTerminal::CompoundExpression,
            expressions,
        ))
    }

    fn handle_expected_actually_error(
        &mut self,
        target_token: Option<Token>,
        expected_tokens: Vec<TokenBase>,
        before_token: Token,
    ) {
        match target_token {
            Some(tk) => {
                self.parse_error.add_error(InvalidSyntax::new(
                    tk.get_token_position(),
                    InvalidSyntaxType::ExpectedNext(ExpectedActuallyTokenPair(expected_tokens, tk)),
                ));
                self.tokens.next();
            }
            None => {
                self.handle_unexpected_eof_error(before_token);
            }
        }
    }

    fn handle_unexpected_eof_error(&mut self, before_token: Token) {
        self.parse_error.add_error(InvalidSyntax::new(
            before_token.get_token_position(),
            InvalidSyntaxType::UnexpectedEof,
        ));
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}

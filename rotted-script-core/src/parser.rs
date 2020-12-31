use std::{todo, vec};

use non_terminal::NonTerminal;
use parse_error::ParseError;
use token_stack::TokenStack;

use crate::lexer::{reserved_word::ReservedWord, token::Token};

use self::ast::Ast;

pub mod ast;
pub(crate) mod ast_type;
pub(crate) mod non_terminal;
mod parse_error;
pub mod token_stack;

pub struct Parser<'a> {
    pub tokens: &'a mut TokenStack<'a>,
    pub ast: Ast,
    #[allow(dead_code)]
    logger: Box<dyn Fn(&str)>,
}

impl<'a> Parser<'a> {
    pub fn new<F>(tokens: &'a mut TokenStack<'a>, logger: &'static F) -> Parser<'a>
    where
        F: Fn(&str),
    {
        Parser {
            tokens,
            ast: Ast::new_node_with_leaves(NonTerminal::TranslationUnit, Vec::new()),
            logger: Box::new(logger),
        }
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        self.parse_translation_unit()
    }

    fn parse_translation_unit(&mut self) -> Result<(), ParseError> {
        while self.tokens.has_next() {
            if let Token::Reserved(r) = self.tokens.look_ahead(1).unwrap() {
                let child = match r {
                    ReservedWord::LeftSquareBracket => self.parse_attribute()?,
                    ReservedWord::Const => self.parse_const_declaration()?,
                    _ => return Err(ParseError::new("unexpected token")),
                };
                self.ast.add_child(child);
            } else {
                return Err(ParseError::new("unexpected token"));
            }
        }
        Ok(())
    }

    fn parse_attribute(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let ast;
        if let Some(Token::Identifier(_)) = self.tokens.look_ahead(1) {
            let attr_token = self.tokens.next().unwrap();
            if self.tokens.next().is_none() {
                return Err(ParseError::new("unexpected eof"));
            }
            ast =
                Ast::new_node_with_leaves(NonTerminal::Attribute, vec![Ast::new_leaf(attr_token)]);
        } else {
            return Err(ParseError::new("unexpected token or eof"));
        }
        Ok(ast)
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
        todo!()
    }
    fn parse_declaration_body(&mut self) -> Result<Ast, ParseError> {
        let ident_ast = if let Some(Token::Identifier(_)) = self.tokens.look_ahead(1) {
            Ast::new_leaf(self.tokens.next().unwrap())
        } else {
            return Err(ParseError::new("unexpected token or eof"));
        };
        self.tokens.consume_reserved(ReservedWord::Equal)?;
        let expr_ast = self.parse_expression()?;
        if let Some(Token::Reserved(ReservedWord::SemiColon)) = self.tokens.next() {
            Ok(Ast::new_node_with_leaves(
                NonTerminal::DeclarationBody,
                vec![ident_ast, expr_ast],
            ))
        } else {
            Err(ParseError::new("unexpected token or eof"))
        }
    }
    fn parse_expression(&mut self) -> Result<Ast, ParseError> {
        Ok(Ast::new_node_with_leaves(
            NonTerminal::Expression,
            vec![match self.tokens.look_ahead(1).unwrap() {
                Token::Identifier(_) => self.parse_call_expression()?,
                Token::String(_) | Token::Number(_) => {
                    let token = self.tokens.next().unwrap();
                    Ast::new_leaf(token)
                }
                Token::Reserved(r) if r == ReservedWord::LeftParenthesis => {
                    // NOTE: after read all of item of inner parentheses, if there is =>, it is arrow expression
                    self.parse_function_expression()?
                }
                _ => panic!(),
            }],
        ))
    }
    fn parse_call_expression(&mut self) -> Result<Ast, ParseError> {
        let mut idents = vec![Ast::new_leaf(self.tokens.next().unwrap())];

        while let Some(Token::Reserved(ReservedWord::Period)) = self.tokens.look_ahead(1) {
            self.tokens.next();
            if let Some(Token::Identifier(_)) = self.tokens.look_ahead(1) {
                idents.push(Ast::new_leaf(self.tokens.next().unwrap()));
            } else {
                return Err(ParseError::new("unexpected token or eof"));
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
            .consume_reserved(ReservedWord::LeftParenthesis)?;
        let mut callers = Vec::new();

        loop {
            if let Some(tk) = self.tokens.look_ahead(1) {
                match tk {
                    Token::String(_)
                    | Token::Number(_)
                    | Token::Identifier(_)
                    | Token::Reserved(ReservedWord::LeftParenthesis) => {
                        callers.push(self.parse_expression()?);
                    }
                    _ => return Err(ParseError::new("unexpected token")),
                }
                if let Some(tk) = self.tokens.look_ahead(1) {
                    match tk {
                        Token::Reserved(ReservedWord::Comma) => {
                            self.tokens.next();
                        }
                        Token::Reserved(ReservedWord::RightParenthesis) => {
                            self.tokens.next();
                            break;
                        }
                        _ => return Err(ParseError::new("unexpected token")),
                    }
                } else {
                    return Err(ParseError::new("unexpected eof"));
                };
            } else {
                return Err(ParseError::new("unexpected eof"));
            }
        }
        Ok(Ast::new_node_with_leaves(NonTerminal::Args, callers))
    }
    fn parse_function_expression(&mut self) -> Result<Ast, ParseError> {
        self.tokens
            .consume_reserved(ReservedWord::LeftParenthesis)?;
        self.tokens
            .consume_reserved(ReservedWord::RightParenthesis)?;
        self.tokens.consume_reserved(ReservedWord::Arrow)?;
        Ok(Ast::new_node_with_leaves(
            NonTerminal::FunctionExpression,
            vec![self.parse_compound_expression()?],
        ))
    }

    fn parse_compound_expression(&mut self) -> Result<Ast, ParseError> {
        let mut expressions = Vec::new();
        self.tokens.consume_reserved(ReservedWord::LeftCurly)?;
        while let Some(tk) = self.tokens.look_ahead(1) {
            let exp = match tk {
                Token::String(_) | Token::Number(_) | Token::Identifier(_) => {
                    self.parse_expression()?
                }
                Token::Reserved(x) if x == ReservedWord::LeftParenthesis => {
                    self.parse_expression()?
                }
                _ => return Err(ParseError::new("unexpected token")),
            };

            let has_semicolon =
                if let Some(Token::Reserved(ReservedWord::SemiColon)) = self.tokens.look_ahead(1) {
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

            if let Some(Token::Reserved(ReservedWord::RightCurly)) = self.tokens.look_ahead(1) {
                break;
            } else if !has_semicolon {
                return Err(ParseError::new("unexpected token"));
            }
        }
        self.tokens.consume_reserved(ReservedWord::RightCurly)?;

        Ok(Ast::new_node_with_leaves(
            NonTerminal::CompoundExpression,
            expressions,
        ))
    }
}

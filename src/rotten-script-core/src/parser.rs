use std::vec;

use non_terminal::NonTerminal;
use parse_error::ParseError;
use token_stack::TokenStack;

use crate::lexer::{reserved_word::ReservedWord, token::TokenBase};

use self::ast::Ast;

pub mod ast;
pub mod ast_type;
pub(crate) mod non_terminal;
mod parse_error;
pub mod token_stack;

mod import_parser;

// struct OptionTokenBase(Option<TokenBase>);
// struct OptionToken(Option<Token>);

// impl PartialEq<OptionTokenBase> for OptionToken {
//     fn eq(&self, other: &OptionTokenBase) -> bool {
//         self.eq(other)
//     }
// }

// impl PartialEq<OptionToken> for OptionTokenBase {
//     fn eq(&self, other: &OptionToken) -> bool {
//         self.eq(other)
//     }
// }

pub struct Parser<'a> {
    pub tokens: &'a mut TokenStack<'a>,
    pub ast: Ast,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut TokenStack<'a>) -> Parser<'a> {
        Parser {
            tokens,
            ast: Ast::new_node_with_leaves(NonTerminal::TranslationUnit, Vec::new()),
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
            if let Some(TokenBase::Reserved(r)) = self.tokens.look_ahead(1) {
                let child = match r {
                    ReservedWord::LeftSquareBracket => {
                        while Some(TokenBase::Reserved(ReservedWord::LeftSquareBracket))
                            == self.tokens.look_ahead(1)
                        {
                            let child = self.parse_attribute()?;
                            self.ast.add_child(child);
                        }
                        if Some(TokenBase::Reserved(ReservedWord::Const))
                            == self.tokens.look_ahead(1)
                        {
                            self.parse_exportable_const_declaration()?
                        } else {
                            return Err(ParseError::new("unexpected eof or token"));
                        }
                    }
                    ReservedWord::Const | ReservedWord::Export => {
                        self.parse_exportable_const_declaration()?
                    }
                    _ => return Err(ParseError::new("unexpected token")),
                };
                self.ast.add_child(child);
            } else {
                return Err(ParseError::new("unexpected eof"));
            }
        }

        Ok(())
    }

    fn parse_attribute(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let ast;
        if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
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

    // ExportableConstDeclaration = [ "export" , ["default"] ] , ConstDeclaration;
    fn parse_exportable_const_declaration(&mut self) -> Result<Ast, ParseError> {
        let mut ast = Vec::new();
        match self.tokens.look_ahead(1) {
            Some(TokenBase::Reserved(r)) => match r {
                ReservedWord::Export => {
                    ast.push(Ast::new_leaf(TokenBase::Reserved(ReservedWord::Export)));
                    self.tokens.next();
                    if let Some(TokenBase::Reserved(tk)) = self.tokens.look_ahead(1) {
                        match tk {
                            ReservedWord::Default => {
                                ast.push(Ast::new_leaf(TokenBase::Reserved(ReservedWord::Default)));
                                self.tokens.next();
                                self.tokens.scan_reserved(ReservedWord::Const)?;
                                ast.push(self.parse_const_declaration()?);
                            }
                            ReservedWord::Const => {
                                ast.push(self.parse_const_declaration()?);
                            }
                            _ => return Err(ParseError::new("unexpected token")),
                        }
                    } else {
                        return Err(ParseError::new("unexpected eof or token"));
                    }
                }
                ReservedWord::Const => {
                    ast.push(self.parse_const_declaration()?);
                }
                _ => return Err(ParseError::new("unexpected token")),
            },
            Some(_) | None => {
                return Err(ParseError::new("unexpected eof or token"));
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
            Ast::new_leaf(self.tokens.next().unwrap())
        } else {
            return Err(ParseError::new("unexpected token or eof"));
        };
        self.tokens.consume_reserved(ReservedWord::Assign)?;
        let expr_ast = self.parse_expression()?;
        if let Some(TokenBase::Reserved(ReservedWord::SemiColon)) = self.tokens.next() {
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
                TokenBase::Identifier(_) => self.parse_call_expression()?,
                TokenBase::String(_) | TokenBase::Number(_) => {
                    let token = self.tokens.next().unwrap();
                    Ast::new_leaf(token)
                }
                TokenBase::Reserved(r) if r == ReservedWord::LeftParenthesis => {
                    // NOTE: after read all of item of inner parentheses, if there is =>, it is arrow expression
                    self.parse_function_expression()?
                }
                _ => panic!(),
            }],
        ))
    }

    fn parse_call_expression(&mut self) -> Result<Ast, ParseError> {
        let mut idents = vec![Ast::new_leaf(self.tokens.next().unwrap())];

        while let Some(TokenBase::Reserved(ReservedWord::Period)) = self.tokens.look_ahead(1) {
            self.tokens.next();
            if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
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

        if self.tokens.look_ahead(1) == Some(TokenBase::Reserved(ReservedWord::RightParenthesis)) {
            self.tokens.next();
            Ok(Ast::new_node_with_leaves(NonTerminal::Args, Vec::new()))
        } else {
            loop {
                if let Some(tk) = self.tokens.look_ahead(1) {
                    match tk {
                        TokenBase::String(_)
                        | TokenBase::Number(_)
                        | TokenBase::Identifier(_)
                        | TokenBase::Reserved(ReservedWord::LeftParenthesis) => {
                            callers.push(self.parse_expression()?);
                        }
                        _ => return Err(ParseError::new("unexpected token")),
                    }
                    if let Some(tk) = self.tokens.look_ahead(1) {
                        match tk {
                            TokenBase::Reserved(ReservedWord::Comma) => {
                                self.tokens.next();
                            }
                            TokenBase::Reserved(ReservedWord::RightParenthesis) => {
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
                TokenBase::String(_) | TokenBase::Number(_) | TokenBase::Identifier(_) => {
                    self.parse_expression()?
                }
                TokenBase::Reserved(x) if x == ReservedWord::LeftParenthesis => {
                    self.parse_expression()?
                }
                _ => return Err(ParseError::new("unexpected token")),
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}

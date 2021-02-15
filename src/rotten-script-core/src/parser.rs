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
                            self.handle_expected_actually_error(
                                self.tokens.nth(1),
                                vec![TokenBase::Reserved(ReservedWord::Const)],
                                self.tokens.peek_token().unwrap(),
                            );
                            // called
                            return Err(ParseError::new("unexpected eof or token"));
                        }
                    }
                    ReservedWord::Const | ReservedWord::Export => {
                        self.parse_exportable_const_declaration()?
                    }
                    _ => {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![
                                TokenBase::Reserved(ReservedWord::LeftSquareBracket),
                                TokenBase::Reserved(ReservedWord::Const),
                                TokenBase::Reserved(ReservedWord::Export),
                            ],
                            self.tokens.peek_token().unwrap(),
                        );
                        // called
                        return Err(ParseError::new("unexpected token"));
                    }
                };
                self.ast.add_child(child);
            } else {
                self.handle_unexpected_eof_error(self.tokens.peek_token().unwrap());
                // called
                return Err(ParseError::new("unexpected eof"));
            }
        }

        Ok(())
    }

    fn parse_attribute(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let ast;
        if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
            let attr_token = self.tokens.next_token().unwrap();
            if self.tokens.next().is_none() {
                self.handle_unexpected_eof_error(self.tokens.peek_token().unwrap());
                // called
                return Err(ParseError::new("unexpected eof"));
            }
            ast =
                Ast::new_node_with_leaves(NonTerminal::Attribute, vec![Ast::new_leaf(attr_token)]);
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::default_identifier()],
                self.tokens.peek_token().unwrap(),
            );
            // called
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
                    ast.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                    if let Some(TokenBase::Reserved(tk)) = self.tokens.look_ahead(1) {
                        match tk {
                            ReservedWord::Default => {
                                ast.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                                // self.tokens.scan_reserved(ReservedWord::Const)?;
                                // // check needs
                                // self.tokens
                                //     .scan_reserved2(ReservedWord::Const)
                                //     .unwrap_or_else(|e| self.parse_error.add_error(e));
                                self.tokens
                                    .scan_reserved2(ReservedWord::Const)
                                    .handle_scan(&mut self.parse_error);
                                // called2
                                ast.push(self.parse_const_declaration()?);
                            }
                            ReservedWord::Const => {
                                ast.push(self.parse_const_declaration()?);
                            }
                            _ => {
                                self.handle_expected_actually_error(
                                    self.tokens.nth(1),
                                    vec![
                                        TokenBase::Reserved(ReservedWord::Default),
                                        TokenBase::Reserved(ReservedWord::Const),
                                    ],
                                    self.tokens.peek_token().unwrap(),
                                );
                                // called
                                return Err(ParseError::new("unexpected token"));
                            }
                        }
                    } else {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![
                                TokenBase::Reserved(ReservedWord::Default),
                                TokenBase::Reserved(ReservedWord::Const),
                            ],
                            self.tokens.peek_token().unwrap(),
                        );
                        // called
                        return Err(ParseError::new("unexpected eof or token"));
                    }
                }
                ReservedWord::Const => {
                    ast.push(self.parse_const_declaration()?);
                }
                _ => {
                    self.handle_expected_actually_error(
                        self.tokens.nth(1),
                        vec![
                            TokenBase::Reserved(ReservedWord::Const),
                            TokenBase::Reserved(ReservedWord::Export),
                        ],
                        self.tokens.peek_token().unwrap(),
                    );
                    // called
                    return Err(ParseError::new("unexpected token"));
                }
            },
            Some(_) | None => {
                self.handle_expected_actually_error(
                    self.tokens.nth(1),
                    vec![
                        TokenBase::Reserved(ReservedWord::Const),
                        TokenBase::Reserved(ReservedWord::Export),
                    ],
                    self.tokens.peek_token().unwrap(),
                );
                // called
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
            Ast::new_leaf(self.tokens.next_token().unwrap())
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::default_identifier()],
                self.tokens.peek_token().unwrap(),
            );
            // called
            return Err(ParseError::new("unexpected token or eof"));
        };
        // self.tokens.consume_reserved(ReservedWord::Assign)?;
        self.tokens
            .consume_reserved2(ReservedWord::Assign)
            .handle_consume(self);
        // called2
        let expr_ast = self.parse_expression()?;
        if let Some(TokenBase::Reserved(ReservedWord::SemiColon)) = self.tokens.look_ahead(1) {
            self.tokens.next();
            Ok(Ast::new_node_with_leaves(
                NonTerminal::DeclarationBody,
                vec![ident_ast, expr_ast],
            ))
        } else {
            self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![TokenBase::Reserved(ReservedWord::SemiColon)],
                self.tokens.peek_token().unwrap(),
            );
            // called
            Err(ParseError::new("unexpected token or eof"))
        }
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
                    vec![TokenBase::Identifier("".to_string())],
                    ne.unwrap(),
                );
                // called
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
        // self.tokens
        //     .consume_reserved(ReservedWord::LeftParenthesis)?;
        // called2
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
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
                        _ => {
                            self.handle_expected_actually_error(
                                self.tokens.nth(1),
                                vec![
                                    TokenBase::String("".to_string()),
                                    TokenBase::Number("".to_string()),
                                    TokenBase::Identifier("".to_string()),
                                    TokenBase::Reserved(ReservedWord::LeftParenthesis),
                                ],
                                self.tokens.peek_token().unwrap(),
                            );
                            // called
                            return Err(ParseError::new("unexpected token"));
                        }
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
                            _ => {
                                self.handle_expected_actually_error(
                                    self.tokens.nth(1),
                                    vec![
                                        TokenBase::Reserved(ReservedWord::Comma),
                                        TokenBase::Reserved(ReservedWord::RightParenthesis),
                                    ],
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
                    };
                } else {
                    self.handle_unexpected_eof_error(self.tokens.peek_token().unwrap());
                    // called
                    return Err(ParseError::new("unexpected eof"));
                }
            }
            Ok(Ast::new_node_with_leaves(NonTerminal::Args, callers))
        }
    }

    fn parse_function_expression(&mut self) -> Result<Ast, ParseError> {
        // self.tokens
        //     .consume_reserved(ReservedWord::LeftParenthesis)?;
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        // called2
        // self.tokens
        //     .consume_reserved(ReservedWord::RightParenthesis)?;
        self.tokens
            .consume_reserved2(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        // called2
        // self.tokens.consume_reserved(ReservedWord::Arrow)?;
        self.tokens
            .consume_reserved2(ReservedWord::Arrow)
            .handle_consume(self);
        // called2
        Ok(Ast::new_node_with_leaves(
            NonTerminal::FunctionExpression,
            vec![self.parse_compound_expression()?],
        ))
    }

    fn parse_compound_expression(&mut self) -> Result<Ast, ParseError> {
        let mut expressions = Vec::new();
        // self.tokens.consume_reserved(ReservedWord::LeftCurly)?;
        self.tokens
            .consume_reserved2(ReservedWord::LeftCurly)
            .handle_consume(self);
        // called2
        while let Some(tk) = self.tokens.look_ahead(1) {
            let exp = match tk {
                TokenBase::String(_) | TokenBase::Number(_) | TokenBase::Identifier(_) => {
                    self.parse_expression()?
                }
                TokenBase::Reserved(ReservedWord::LeftParenthesis) => self.parse_expression()?,
                _ => {
                    self.handle_expected_actually_error(
                        self.tokens.nth(1),
                        vec![
                            TokenBase::default_string(),
                            TokenBase::default_number(),
                            TokenBase::default_identifier(),
                            TokenBase::Reserved(ReservedWord::LeftParenthesis),
                        ],
                        self.tokens.peek_token().unwrap(),
                    );
                    // called
                    return Err(ParseError::new("unexpected token"));
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
                // called
                return Err(ParseError::new("unexpected token"));
            }
        }
        // self.tokens.consume_reserved(ReservedWord::RightCurly)?;
        self.tokens
            .consume_reserved2(ReservedWord::RightCurly)
            .handle_consume(self);
        // called2

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

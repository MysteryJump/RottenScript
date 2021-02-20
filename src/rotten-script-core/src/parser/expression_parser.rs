use super::{
    ast::Ast, non_terminal::NonTerminal, parse_error::ParseError, InvalidSyntaxResultHandler,
    Parser,
};
use crate::lexer::{reserved_word::ReservedWord, token::TokenBase};

macro_rules! parse_left_assoc_binary_expr {
    ($self:ident, $current_type:path , $next_func:ident, $($tokens:pat),+ ) => {
        let mut astc = Vec::new();
        let first = $self.$next_func()?;
        // astc.push($self.$next_func()?);
        loop {
            match $self.tokens.look_ahead(1) {
                $(
                    Some($tokens) => {
                        astc.push(Ast::new_leaf($self.tokens.next_token().unwrap()));
                        astc.push($self.$next_func()?);
                    }
                )+
                _ => break,
            }
        }

        if astc.is_empty() {
            return Ok(first);
        }
        let mut second = Vec::new();
        second.push(first);
        second.append(&mut astc);
        return Ok(Ast::new_node_with_leaves($current_type, second));
    };
}

impl<'a> Parser<'a> {
    // pub fn parse_expression(&mut self) -> Result<Ast, ParseError> {
    //     Ok(Ast::new_node_with_leaves(
    //         NonTerminal::Expression,
    //         vec![match self.tokens.look_ahead(1).unwrap() {
    //             // conflicting call expression and identifier
    //             TokenBase::Identifier(_) => self.parse_call_expression()?,
    //             TokenBase::String(_)
    //             | TokenBase::Number(_)
    //             | TokenBase::Reserved(ReservedWord::True)
    //             | TokenBase::Reserved(ReservedWord::False) => {
    //                 Ast::new_leaf(self.tokens.next_token().unwrap())
    //             }
    //             // conflicting function expression and parenthesized expression
    //             TokenBase::Reserved(ReservedWord::LeftParenthesis) => {
    //                 // NOTE: after read all of item of inner parentheses, if there is =>, it is arrow expression
    //                 self.parse_function_expression()?
    //             }
    //             _ => panic!(),
    //         }],
    //     ))
    // }

    // fn parse_call_expression(&mut self) -> Result<Ast, ParseError> {
    //     let mut idents = vec![Ast::new_leaf(self.tokens.next_token().unwrap())];

    //     while let Some(TokenBase::Reserved(ReservedWord::Dot)) = self.tokens.look_ahead(1) {
    //         let ne = self.tokens.next_token();
    //         if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
    //             idents.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
    //         } else {
    //             let tk = self.tokens.next_token();
    //             self.handle_expected_actually_error(
    //                 tk,
    //                 vec![TokenBase::default_identifier()],
    //                 ne.unwrap(),
    //             );
    //         }
    //     }

    //     let args = self.parse_args()?;
    //     idents.push(args);
    //     Ok(Ast::new_node_with_leaves(
    //         NonTerminal::CallExpression,
    //         idents,
    //     ))
    // }

    pub fn parse_expression2(&mut self) -> Result<Ast, ParseError> {
        self.parse_logical_or_expression()
    }

    fn parse_logical_or_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::LogicalOrExpression,
            parse_logical_and_expression,
            TBR!("||")
        );
    }

    fn parse_logical_and_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::LogicalAndExpression,
            parse_bitwise_or_expression,
            TBR!("&&")
        );
    }

    fn parse_bitwise_or_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::BitwiseOrExpression,
            parse_bitwise_xor_expression,
            TBR!("|")
        );
    }

    fn parse_bitwise_xor_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::BitwiseXorExpression,
            parse_bitwise_and_expression,
            TBR!("^")
        );
    }

    fn parse_bitwise_and_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::BitwiseAndExpression,
            parse_equality_expression,
            TBR!("&")
        );
    }

    fn parse_equality_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::EqualityExpression,
            parse_relational_expression,
            TBR!("=="),
            TBR!("!=")
        );
    }

    fn parse_relational_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::RelationalExpression,
            parse_shift_expression,
            TBR!(">"),
            TBR!("<"),
            TBR!("<="),
            TBR!(">=")
        );
    }

    fn parse_shift_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::ShiftExpression,
            parse_additive_expression,
            TBR!("<<"),
            TBR!(">>"),
            TBR!(">>>")
        );
    }

    fn parse_additive_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::AdditiveExpression,
            parse_multiplicative_expression,
            TBR!("-"),
            TBR!("+")
        );
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Ast, ParseError> {
        parse_left_assoc_binary_expr!(
            self,
            NonTerminal::MultiplicativeExpression,
            parse_exponential_expression,
            TBR!("*"),
            TBR!("/"),
            TBR!("%")
        );
    }

    pub fn parse_exponential_expression(&mut self) -> Result<Ast, ParseError> {
        let mut astc = Vec::new();
        match self.tokens.look_ahead(1) {
            Some(TBR!("!")) | Some(TBR!("~")) | Some(TBR!("+")) | Some(TBR!("-")) => {
                astc.push(self.parse_unary_expression()?);
                if let Some(TBR!("**")) = self.tokens.look_ahead(1) {
                    self.handle_exponential_error(self.tokens.nth(1).unwrap());
                    self.tokens.next();
                    astc.push(self.parse_exponential_expression()?);
                }
            }
            _ => {
                let primary = self.parse_primary_expression()?;
                if let Some(TBR!("**")) = self.tokens.look_ahead(1) {
                    astc.push(primary);
                    self.tokens.next();
                    astc.push(self.parse_exponential_expression()?);
                } else {
                    astc.push(primary);
                }
            }
        }
        if astc.len() == 1 {
            let f = astc.remove(0);
            Ok(f)
        } else {
            Ok(Ast::new_node_with_leaves(
                NonTerminal::ExponentiationExpression,
                astc,
            ))
        }
    }

    pub fn parse_unary_expression(&mut self) -> Result<Ast, ParseError> {
        match self.tokens.look_ahead(1) {
            Some(TBR!("!")) | Some(TBR!("~")) | Some(TBR!("+")) | Some(TBR!("-")) => {
                let tk_ast = Ast::new_leaf(self.tokens.next_token().unwrap());
                Ok(Ast::new_node_with_leaves(
                    NonTerminal::UnaryExpression,
                    vec![tk_ast, self.parse_unary_expression()?],
                ))
            }
            _ => self.parse_primary_expression(),
        }
    }

    pub fn parse_primary_expression(&mut self) -> Result<Ast, ParseError> {
        let mut asts = Vec::new();

        match self.tokens.look_ahead(1) {
            // Literal or Identifier
            Some(TokenBase::String(_))
            | Some(TokenBase::Number(_))
            | Some(TokenBase::Identifier(_)) => {
                asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
            }
            // Function or Parenthesized
            Some(TokenBase::Reserved(ReservedWord::LeftParenthesis)) => {
                if self.should_continue_as_function_expr()? {
                    asts.push(self.parse_function_expression()?);
                } else {
                    asts.push(self.parse_parenthesized_expression()?);
                };
            }
            // Compound
            Some(TokenBase::Reserved(ReservedWord::LeftCurly)) => {
                asts.push(self.parse_compound_expression()?);
            }
            None | Some(_) => self.handle_expected_actually_error(
                self.tokens.nth(1),
                vec![
                    TokenBase::default_string(),
                    TokenBase::default_number(),
                    TokenBase::default_identifier(),
                    TokenBase::Reserved(ReservedWord::LeftParenthesis),
                    TokenBase::Reserved(ReservedWord::LeftCurly),
                ],
                self.tokens.peek_token().unwrap(),
            ),
        }

        while let Some(tk) = self.tokens.look_ahead(1) {
            match tk {
                TokenBase::Reserved(ReservedWord::Dot) => {
                    while self.tokens.look_ahead(1) == Some(TokenBase::Reserved(ReservedWord::Dot))
                    {
                        // NOTE: Add dot(.) for future (e.g. optional chaining)
                        asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                        if let Some(TokenBase::Identifier(_)) = self.tokens.look_ahead(1) {
                            asts.push(Ast::new_leaf(self.tokens.next_token().unwrap()));
                        } else {
                            self.handle_expected_actually_error(
                                self.tokens.nth(1),
                                vec![TokenBase::default_identifier()],
                                self.tokens.peek_token().unwrap(),
                            )
                        }
                    }
                }
                TokenBase::Reserved(ReservedWord::LeftParenthesis) => {
                    while self.tokens.look_ahead(1)
                        == Some(TokenBase::Reserved(ReservedWord::LeftParenthesis))
                    {
                        asts.push(self.parse_args()?);
                    }
                }
                // _ => self.handle_expected_actually_error(
                //     self.tokens.nth(1),
                //     vec![
                //         TokenBase::Reserved(ReservedWord::Dot),
                //         TokenBase::Reserved(ReservedWord::LeftParenthesis),
                //     ],
                //     self.tokens.peek_token().unwrap(),
                // ),
                _ => {
                    break;
                }
            }
        }

        Ok(Ast::new_node_with_leaves(
            NonTerminal::PrimaryExpression,
            asts,
        ))
    }

    fn parse_function_expression(&mut self) -> Result<Ast, ParseError> {
        self.tokens
            .consume_reserved(ReservedWord::LeftParenthesis)
            .handle_consume(self);
        self.tokens
            .consume_reserved(ReservedWord::RightParenthesis)
            .handle_consume(self);
        self.tokens
            .consume_reserved(ReservedWord::Arrow)
            .handle_consume(self);
        Ok(Ast::new_node_with_leaves(
            NonTerminal::FunctionExpression,
            vec![self.parse_compound_expression()?],
        ))
    }

    fn parse_compound_expression(&mut self) -> Result<Ast, ParseError> {
        let mut expressions = Vec::new();
        self.tokens
            .consume_reserved(ReservedWord::LeftCurly)
            .handle_consume(self);
        while let Some(tk) = self.tokens.look_ahead(1) {
            match tk {
                TokenBase::String(_)
                | TokenBase::Number(_)
                | TokenBase::Identifier(_)
                | TokenBase::Reserved(ReservedWord::LeftParenthesis) => {
                    let exp = self.parse_expression2()?;
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

                    if let Some(TokenBase::Reserved(ReservedWord::RightCurly)) =
                        self.tokens.look_ahead(1)
                    {
                        break;
                    } else if !has_semicolon {
                        self.handle_expected_actually_error(
                            self.tokens.nth(1),
                            vec![TokenBase::Reserved(ReservedWord::SemiColon)],
                            self.tokens.peek_token().unwrap(),
                        );
                    }
                }
                TokenBase::Reserved(ReservedWord::Const) => {
                    expressions.push(self.parse_const_declaration()?)
                }
                TokenBase::Reserved(ReservedWord::Let) => {
                    expressions.push(self.parse_let_declaration()?)
                }
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
                    expressions.push(Ast::new_leaf(target_token.unwrap()))
                }
            }
        }
        self.tokens
            .consume_reserved(ReservedWord::RightCurly)
            .handle_consume(self);

        Ok(Ast::new_node_with_leaves(
            NonTerminal::CompoundExpression,
            expressions,
        ))
    }

    fn parse_parenthesized_expression(&mut self) -> Result<Ast, ParseError> {
        self.tokens.next();
        let ast = self.parse_expression2()?;
        self.tokens
            .consume_reserved(ReservedWord::RightParenthesis)
            .handle_consume(self);
        Ok(Ast::new_node_with_leaves(
            NonTerminal::ParenthesizedExpression,
            vec![ast],
        ))
    }

    fn should_continue_as_function_expr(&self) -> Result<bool, ParseError> {
        let mut count = 1;
        let mut depth = 0;
        loop {
            match self.tokens.look_ahead(count) {
                Some(TokenBase::Reserved(ReservedWord::LeftParenthesis)) => {
                    depth += 1;
                }
                Some(TokenBase::Reserved(ReservedWord::RightParenthesis)) => {
                    depth -= 1;
                }
                Some(_) => {}
                None => return Err(ParseError::new("unexpected eof")),
            }
            count += 1;
            if depth == 0 {
                return if self.tokens.look_ahead(count)
                    == Some(TokenBase::Reserved(ReservedWord::Arrow))
                {
                    Ok(true)
                } else {
                    Ok(false)
                };
            }
        }
    }
}

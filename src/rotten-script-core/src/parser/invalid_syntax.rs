use std::{collections::HashSet, fmt::Display};

use colored::Colorize;

use crate::lexer::{
    reserved_word::ReservedWord,
    token::{Token, TokenBase, TokenPosition},
};

#[derive(Debug)]
pub struct InvalidSyntax {
    position: TokenPosition,
    invalid_syntax_type: InvalidSyntaxType,
}

#[derive(Debug)]
pub enum InvalidSyntaxType {
    ExpectedNext(ExpectedActuallyTokenPair),
    UnexpectedEof,
    ExponentialError(),
}

#[derive(Debug)]
pub struct ExpectedActuallyTokenPair(pub Vec<TokenBase>, pub Token);

impl InvalidSyntax {
    pub fn new(position: TokenPosition, invalid_syntax_type: InvalidSyntaxType) -> Self {
        Self {
            position,
            invalid_syntax_type,
        }
    }
    pub fn get_type(&self) -> &InvalidSyntaxType {
        &self.invalid_syntax_type
    }
}

impl Display for InvalidSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}\n\t --> {}:{}:{}",
            "error".red().bold(),
            self.invalid_syntax_type,
            self.position.path,
            self.position.ln,
            self.position.col
        )
    }
}

impl Display for InvalidSyntaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidSyntaxType::ExpectedNext(pair) => {
                write!(
                    f,
                    "expected {}, found `{}`",
                    token_base_array_to_string(&pair.0),
                    pair.1
                )
            }
            InvalidSyntaxType::UnexpectedEof => {
                write!(f, "unexpected EOF")
            }
            InvalidSyntaxType::ExponentialError() => {
                write!(f, "cannot allow exponential operator (**) after the unary operated expression (e.g. +1 ** 2), 
                you need to use parenthesized unary operator (e.g. (+1) ** 2)")
            }
        }
    }
}

fn token_base_array_to_string(arr: &[TokenBase]) -> String {
    arr.iter()
        .map(|x| match x {
            TokenBase::String(_)
            | TokenBase::Number(_)
            | TokenBase::Reserved(ReservedWord::True)
            | TokenBase::Reserved(ReservedWord::False) => "literal".to_string(),
            TokenBase::Reserved(r) => format!("`{}`", r),
            TokenBase::Identifier(_) => "identifier".to_string(),
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ")
}

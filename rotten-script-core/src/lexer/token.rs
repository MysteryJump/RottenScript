use std::fmt::Display;

use reserved_word::ReservedWord;

use super::reserved_word;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    String(String),
    Number(String),
    Reserved(ReservedWord),
    Identifier(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::String(s) => write!(f, "string({})", s),
            Token::Number(n) => write!(f, "number({})", n),
            Token::Reserved(r) => write!(f, "{:?}", r),
            Token::Identifier(i) => write!(f, "identifier({})", i),
        }
    }
}

use std::{fmt::Display, write};

#[derive(Debug)]
pub struct ParseError {
    pub message: &'static str,
}

impl ParseError {
    pub fn new(message: &'static str) -> ParseError {
        ParseError { message }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error: {}", self.message)
    }
}

use std::{error::Error, fmt::Display, write};

use super::invalid_syntax::{InvalidSyntax, InvalidSyntaxType};

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

impl Error for ParseError {}

#[derive(Debug)]
pub struct ParseError2 {
    errors: Vec<InvalidSyntax>,
}

impl Default for ParseError2 {
    fn default() -> Self {
        Self { errors: Vec::new() }
    }
}

impl ParseError2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn add_error(&mut self, invalid_syntax: InvalidSyntax) {
        self.errors.push(invalid_syntax);
    }
}

impl Display for ParseError2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut has_eof_error = false;
        for error in &self.errors {
            if has_eof_error {
                continue;
            }
            if let InvalidSyntaxType::UnexpectedEof = error.get_type() {
                has_eof_error = true;
            }

            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl Error for ParseError2 {}

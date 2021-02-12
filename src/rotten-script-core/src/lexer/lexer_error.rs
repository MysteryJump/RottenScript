use std::fmt::Display;

use colored::Colorize;

use super::token::Token;
#[derive(Debug)]
pub struct LexerError {
    invalid_tokens: Vec<Token>,
}

impl LexerError {
    pub fn new(invalid_tokens: Vec<Token>) -> Self {
        Self { invalid_tokens }
    }
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in &self.invalid_tokens {
            let base_text = token.get_base_text();
            let position = token.get_token_position();
            f.write_str(&format!(
                "{}: invalid token {} \n\t --> {}:{}:{}\n",
                "error".red().bold(),
                base_text,
                position.path,
                position.ln,
                position.col
            ))?;
        }
        Ok(())
    }
}

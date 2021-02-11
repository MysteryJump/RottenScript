use std::sync::Mutex;

use once_cell::sync::Lazy;

pub mod builder;
pub mod lexer;
pub mod parser;
pub mod semantic_analyzer;

pub struct Logger<'a> {
    pub logger: Option<Box<dyn Fn(&'a str) + Sync + Send>>,
}

impl<'a> Logger<'a> {
    pub fn log(&self, input: &'a str) {
        if let Some(f) = self.logger.as_ref() {
            f(input)
        }
    }
}

pub static LOGGER: Lazy<Mutex<Logger>> = Lazy::new(|| Mutex::new(Logger { logger: None }));

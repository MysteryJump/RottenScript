use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let res = format!($($arg)*);
        crate::LOGGER.clone().lock().unwrap().log(&res);
    }}
}

pub mod builder;
pub mod lexer;
pub mod parser;
pub mod semantic_analyzer;

pub struct Logger {
    pub logger: Option<Box<dyn Fn(String) + Sync + Send>>,
}

impl Logger {
    pub fn log(&self, input: &str) {
        if let Some(f) = self.logger.as_ref() {
            f(input.to_string())
        }
    }
}

pub static LOGGER: Lazy<Arc<Mutex<Logger>>> =
    Lazy::new(|| Arc::new(Mutex::new(Logger { logger: None })));

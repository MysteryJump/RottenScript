use std::{fmt::Display, rc::Rc};

use reserved_word::ReservedWord;

use super::reserved_word;

#[derive(Clone, PartialEq)]
pub enum TokenBase {
    String(String),
    Number(String),
    Reserved(ReservedWord),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    token: Option<TokenBase>,
    base_str: String,
    ln: u32,
    col: u32,
    ind: u64,
    len: usize,
    file_path: Rc<String>,
}

#[derive(Debug)]
pub struct TokenPosition {
    pub ln: u32,
    pub col: u32,
    pub ind: u64,
    pub len: usize,
    pub path: Rc<String>,
}

impl Token {
    pub fn new(
        token_base: Result<TokenBase, String>,
        ln: u32,
        col: u32,
        ind: u64,
        file_path: Rc<String>,
    ) -> Self {
        let base_str = match &token_base {
            Ok(tb) => tb.to_string(),
            Err(bs) => bs.to_string(),
        };
        let len = base_str.len();
        Self {
            token: match &token_base {
                Ok(tb) => Some(tb.clone()),
                Err(_) => None,
            },
            base_str,
            ln,
            col,
            ind,
            len,
            file_path,
        }
    }

    pub fn get_token(&self) -> &Option<TokenBase> {
        &self.token
    }

    pub fn get_token_position(&self) -> TokenPosition {
        TokenPosition {
            ln: self.ln,
            col: self.col,
            ind: self.ind,
            len: self.len,
            path: self.file_path.clone(),
        }
    }

    pub fn get_base_text(&self) -> &str {
        &self.base_str
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token {
            Some(tk) => write!(f, "{}", tk),
            None => write!(f, "[unknown token: {}]", self.base_str),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match &self.token {
            Some(tk) => {
                if other.token.is_none() {
                    false
                } else {
                    tk == other.token.as_ref().unwrap()
                }
            }
            None => false,
        }
    }
}

impl Display for TokenBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenBase::String(s) => write!(f, "\"{}\"", s),
            TokenBase::Number(n) => write!(f, "{}", n),
            TokenBase::Reserved(r) => write!(f, "{}", r),
            TokenBase::Identifier(i) => write!(f, "{}", i),
        }
    }
}

impl std::fmt::Debug for TokenBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenBase::String(s) => write!(f, "string({})", s),
            TokenBase::Number(n) => write!(f, "number({})", n),
            TokenBase::Reserved(r) => write!(f, "{:?}", r),
            TokenBase::Identifier(i) => write!(f, "identifier({})", i),
        }
    }
}

impl Eq for TokenBase {}

impl PartialEq<TokenBase> for Token {
    fn eq(&self, other: &TokenBase) -> bool {
        match self.get_token() {
            Some(tb) => other == tb,
            None => false,
        }
    }
}

impl PartialEq<Token> for TokenBase {
    fn eq(&self, other: &Token) -> bool {
        match other.get_token() {
            Some(tb) => self == tb,
            None => false,
        }
    }
}

impl TokenBase {
    pub fn default_string() -> TokenBase {
        TokenBase::String(String::new())
    }
    pub fn default_number() -> TokenBase {
        TokenBase::Number(String::new())
    }
    pub fn default_identifier() -> TokenBase {
        TokenBase::Identifier(String::new())
    }
    pub fn get_literal_token_bases() -> [TokenBase; 4] {
        [
            TokenBase::default_string(),
            TokenBase::default_number(),
            TokenBase::Reserved(ReservedWord::True),
            TokenBase::Reserved(ReservedWord::False),
        ]
    }
}

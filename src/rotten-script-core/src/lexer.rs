pub mod reserved_word;
pub mod token;

use regex::Regex;
use reserved_word::ReservedWord;
use token::Token;

pub struct Lexer<'a> {
    source: &'a str,
    pub tokens: Vec<Token>,
    #[allow(dead_code)]
    logger: Box<dyn Fn(&str)>,
}

impl<'a> Lexer<'a> {
    pub fn new<F>(code: &'a str, logger: &'static F) -> Lexer<'a>
    where
        F: Fn(&str),
    {
        Lexer {
            source: code,
            tokens: Vec::new(),
            logger: Box::new(logger),
        }
    }

    pub fn lex(&mut self) -> Result<(), String> {
        let reserved_regex = Regex::new(r"^(=[>]?|\(|\)|\{|\}|\[|\]|\.|,|;|const|let)").unwrap();
        let identifier_regex = Regex::new(r"^([_a-zA-Z][_a-zA-Z0-9]*)").unwrap();
        let dq_str_literal_regex = Regex::new(r#"^"(.*)""#).unwrap();
        let sq_str_literal_regex = Regex::new(r"^'(.*)'#").unwrap();
        let mut line_com_mode = false;
        let mut code = String::from(self.source);

        while !code.is_empty() {
            let replace_length;
            if line_com_mode {
                if let Some(last_ind) = code.find(|x| x == '\r' || x == '\n' || x == '\0') {
                    replace_length = last_ind as i32 + 1;
                    line_com_mode = false;
                } else {
                    break;
                }
            } else if code.starts_with(&['\n', '\r', ' ', '\t'][..]) {
                replace_length = 1;
            } else if let Some(reserveds) = reserved_regex.find(&code) {
                let mat = reserveds.as_str();

                let word = match mat {
                    "=>" => ReservedWord::Arrow,
                    "=" => ReservedWord::Equal,
                    "(" => ReservedWord::LeftParenthesis,
                    ")" => ReservedWord::RightParenthesis,
                    "[" => ReservedWord::LeftSquareBracket,
                    "]" => ReservedWord::RightSquareBracket,
                    "{" => ReservedWord::LeftCurly,
                    "}" => ReservedWord::RightCurly,
                    "." => ReservedWord::Period,
                    "," => ReservedWord::Comma,
                    ";" => ReservedWord::SemiColon,
                    "const" => ReservedWord::Const,
                    "let" => ReservedWord::Let,
                    _ => return Err(String::from("some error")),
                };
                self.tokens.push(Token::Reserved(word));
                replace_length = mat.len() as i32;
            } else if let Some(ident) = identifier_regex.find(&code) {
                let mat = ident.as_str();
                self.tokens.push(Token::Identifier(mat.to_string()));
                replace_length = mat.len() as i32;
            } else if code.starts_with("//") {
                line_com_mode = true;
                replace_length = 2;
            } else if let Some(dq_str) = dq_str_literal_regex.find(&code) {
                let mat = dq_str.as_str().trim_matches('"');
                self.tokens.push(Token::String(mat.to_string()));
                replace_length = mat.len() as i32 + 2;
            } else if let Some(sq_str) = sq_str_literal_regex.find(&code) {
                let mat = sq_str.as_str().trim_matches('\'');
                self.tokens.push(Token::String(mat.to_string()));
                replace_length = mat.len() as i32 + 2;
            } else {
                return Err(String::from("found unknown token"));
            }
            if replace_length >= 0 {
                code.replace_range(0..replace_length as usize, "");
            }
        }
        Ok(())
    }
}

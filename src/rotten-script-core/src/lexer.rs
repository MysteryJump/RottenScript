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
        let reserved_regex =
            Regex::new(r"^(=[>]?|\(|\)|\{|\}|\[|\]|\.|,|;|const|let|import|export|from|default)")
                .unwrap();
        let identifier_regex = Regex::new(r"^([_a-zA-Z][_a-zA-Z0-9]*)").unwrap();
        let dq_str_literal_regex = Regex::new(r#"^"(.*?)""#).unwrap();
        let sq_str_literal_regex = Regex::new(r"^'(.*?)'").unwrap();
        let number_literal_regex = Regex::new(r"^(\.\d+|[1-9]\d*\.\d+|[1-9]\d*|0\.\d*|0)").unwrap();
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
            } else if let Some(number) = number_literal_regex.find(&code) {
                let mat = number.as_str();
                self.tokens.push(Token::Number(mat.to_string()));
                replace_length = mat.len() as i32;
            } else if let Some(reserveds) = reserved_regex.find(&code) {
                let mat = reserveds.as_str();

                let word = match mat {
                    "=>" => ReservedWord::Arrow,
                    "=" => ReservedWord::Assign,
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
                    "import" => ReservedWord::Import,
                    "export" => ReservedWord::Export,
                    "from" => ReservedWord::From,
                    "default" => ReservedWord::Default,
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

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn lexer_single_comment_test() {
        let cases = vec![
            "// test1",
            "//test3\n",
            "//test4\r\n",
            "\n//ttest\n//te",
            "\n//\r\n",
        ];
        for item in cases {
            let mut lexer = Lexer::new(item, &logger);
            lexer.lex().unwrap();
            assert_eq!(0, lexer.tokens.len());
        }
    }
    #[test]
    fn identifier_test() {
        let cases = vec!["ident", "ident ident", "ide\nnt", "ode \t den"];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, &logger);
            lexer.lex().unwrap();

            if ind == 0 {
                assert_eq!(1, lexer.tokens.len());
            } else {
                let expected_idents = item
                    .split(|x| x == ' ' || x == '\n' || x == '\t')
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>();
                assert_eq!(lexer.tokens.len(), 2);
                assert_eq!(
                    lexer.tokens[0],
                    Token::Identifier(expected_idents[0].to_string())
                );
                assert_eq!(
                    lexer.tokens[1],
                    Token::Identifier(expected_idents[1].to_string())
                );
            }
        }
    }

    #[test]
    fn sq_string_test() {
        let cases = vec!["'test1'", "'test3''yrdy'", "'tes\nyr'", "'test\"te'"];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, &logger);
            if lexer.lex().is_ok() {
                match ind {
                    0 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test1")), lexer.tokens[0]);
                    }
                    1 => {
                        assert_eq!(2, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test3")), lexer.tokens[0]);
                        assert_eq!(Token::String(String::from("yrdy")), lexer.tokens[1]);
                    }
                    3 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test\"te")), lexer.tokens[0]);
                    }
                    _ => panic!(),
                }
            }
        }
    }

    #[test]
    fn dq_string_test() {
        let cases = vec![
            r#""test1""#,
            r#""test3""yrdy""#,
            "\"tes\nyr\"",
            r#""test'te""#,
        ];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, &logger);
            if lexer.lex().is_ok() {
                match ind {
                    0 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test1")), lexer.tokens[0]);
                    }
                    1 => {
                        assert_eq!(2, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test3")), lexer.tokens[0]);
                        assert_eq!(Token::String(String::from("yrdy")), lexer.tokens[1]);
                    }
                    3 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq!(Token::String(String::from("test'te")), lexer.tokens[0]);
                    }
                    _ => panic!(),
                }
            }
        }
    }
    #[test]
    fn number_test() {
        let valid_cases = vec!["33", ".435", "3232.042", "0", "0.33"];
        for (ind, item) in valid_cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, &logger);
            lexer.lex().unwrap();
            match ind {
                0 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq!(Token::Number("33".to_string()), lexer.tokens[0]);
                }
                1 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq!(Token::Number(".435".to_string()), lexer.tokens[0]);
                }
                2 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq!(Token::Number("3232.042".to_string()), lexer.tokens[0]);
                }
                3 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq!(Token::Number("0".to_string()), lexer.tokens[0]);
                }
                4 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq!(Token::Number("0.33".to_string()), lexer.tokens[0]);
                }
                _ => panic!(),
            }
        }
    }
    #[test]
    fn reserved_test() {
        let cases = vec![
            "=", "(", ")", "{", "}", "[", "]", ".", ",", ";", "=>", "const", "let", "import",
            "export", "default", "from",
        ];
        use super::ReservedWord::*;
        use super::Token::Reserved;
        for (ind, case) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(case, &logger);
            lexer.lex().unwrap();
            assert_eq!(1, lexer.tokens.len());
            let first = lexer.tokens[0].clone();
            match ind {
                0 => assert_eq!(Reserved(Assign), first),
                1 => assert_eq!(Reserved(LeftParenthesis), first),
                2 => assert_eq!(Reserved(RightParenthesis), first),
                3 => assert_eq!(Reserved(LeftCurly), first),
                4 => assert_eq!(Reserved(RightCurly), first),
                5 => assert_eq!(Reserved(LeftSquareBracket), first),
                6 => assert_eq!(Reserved(RightSquareBracket), first),
                7 => assert_eq!(Reserved(Period), first),
                8 => assert_eq!(Reserved(Comma), first),
                9 => assert_eq!(Reserved(SemiColon), first),
                10 => assert_eq!(Reserved(Arrow), first),
                11 => assert_eq!(Reserved(Const), first),
                12 => assert_eq!(Reserved(Let), first),
                13 => assert_eq!(Reserved(Import), first),
                14 => assert_eq!(Reserved(Export), first),
                15 => assert_eq!(Reserved(Default), first),
                16 => assert_eq!(Reserved(From), first),
                _ => panic!(),
            }
        }
    }

    fn logger(_line: &str) {}
}

mod lex_error;
pub mod reserved_word;
pub mod token;

use std::rc::Rc;

use lex_error::LexError;
use regex::Regex;
use reserved_word::ReservedWord;
use token::{Token, TokenBase};

pub struct Lexer<'a> {
    source: &'a str,
    pub tokens: Vec<Token>,
    ind: u64,
    col: u32,
    ln: u32,
    file_path: Rc<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str, path: &'a str) -> Lexer<'a> {
        Lexer {
            source: code,
            tokens: Vec::new(),
            ind: 0,
            col: 1,
            ln: 1,
            file_path: Rc::new(path.to_string()),
        }
    }

    fn push_token(&mut self, token_base: TokenBase) {
        self.tokens.push(Token::new(
            Ok(token_base),
            self.ln,
            self.col,
            self.ind,
            self.file_path.clone(),
        ));
    }

    fn push_invalid_token(&mut self, base_str: String) -> Token {
        let tk = Token::new(
            Err(base_str),
            self.ln,
            self.col,
            self.ind,
            self.file_path.clone(),
        );
        self.tokens.push(tk.clone());
        tk
    }

    pub fn lex(&mut self) -> Result<(), LexError> {
        let reserved_regex = Regex::new(
            r"^(={1,2}[>]?|\(|\)|\{|\}|\[|\]|\.|,|:|;|\+=?|\*{1,2}=?|/=?|-=?|%=?|<<?=?|>{1,3}=?|&&|&=?|\|\||\|=?|\^=?|\~|!=?|const|let|import|export|from|default|true|false)",
        )
        .unwrap();
        let identifier_regex = Regex::new(r"^([_a-zA-Z][_a-zA-Z0-9]*)").unwrap();
        let dq_str_literal_regex = Regex::new(r#"^"(.*?)""#).unwrap();
        let sq_str_literal_regex = Regex::new(r"^'(.*?)'").unwrap();
        let number_literal_regex = Regex::new(r"^(\.\d+|[1-9]\d*\.\d+|[1-9]\d*|0\.\d*|0)").unwrap();
        let mut line_com_mode = false;
        let mut code = String::from(self.source);
        let mut invalid_tokens = Vec::new();

        while !code.is_empty() {
            let replace_length;
            if line_com_mode {
                if let Some(last_ind) = code.find(|x| x == '\r' || x == '\n' || x == '\0') {
                    replace_length = last_ind + 1;
                    line_com_mode = false;
                    self.ln += 1;
                    self.col = 1;
                } else {
                    break;
                }
            } else if code.starts_with(&['\n', '\r', ' ', '\t'][..]) {
                if code.starts_with('\n') {
                    self.ln += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }

                replace_length = 1;
            } else if code.starts_with("//") {
                line_com_mode = true;
                replace_length = 2;
                self.col += replace_length as u32;
            } else if let Some(number) = number_literal_regex.find(&code) {
                let mat = number.as_str();

                self.push_token(TokenBase::Number(mat.to_string()));
                replace_length = mat.len();
                self.col += replace_length as u32;
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
                    "." => ReservedWord::Dot,
                    "," => ReservedWord::Comma,
                    ";" => ReservedWord::SemiColon,
                    "+" => ReservedWord::Add,
                    "*" => ReservedWord::Mult,
                    "/" => ReservedWord::Div,
                    "-" => ReservedWord::Sub,
                    "%" => ReservedWord::Mod,
                    "<" => ReservedWord::Less,
                    ">" => ReservedWord::Greater,
                    "&" => ReservedWord::And,
                    "|" => ReservedWord::Or,
                    "^" => ReservedWord::Xor,
                    "~" => ReservedWord::Not,
                    "!" => ReservedWord::LogicalNot,
                    ":" => ReservedWord::Colon,
                    "const" => ReservedWord::Const,
                    "let" => ReservedWord::Let,
                    "import" => ReservedWord::Import,
                    "export" => ReservedWord::Export,
                    "from" => ReservedWord::From,
                    "default" => ReservedWord::Default,
                    "true" => ReservedWord::True,
                    "false" => ReservedWord::False,
                    "<<" => ReservedWord::LeftShift,
                    ">>" => ReservedWord::RightShift,
                    ">>>" => ReservedWord::UnsignedRightShift,
                    "<=" => ReservedWord::LessOrEq,
                    ">=" => ReservedWord::GreaterOrEq,
                    "==" => ReservedWord::Equal,
                    "!=" => ReservedWord::NotEqual,
                    "**" => ReservedWord::Exponential,
                    "&&" => ReservedWord::LogicalAnd,
                    "||" => ReservedWord::LogicalOr,
                    "+=" => ReservedWord::AdditiveAssign,
                    "-=" => ReservedWord::SubtractiveAssign,
                    "*=" => ReservedWord::MultiplicativeAssign,
                    "/=" => ReservedWord::DivisiveAssign,
                    "%=" => ReservedWord::ModuloAssign,
                    "<<=" => ReservedWord::LeftShiftAssign,
                    ">>=" => ReservedWord::RightShiftAssign,
                    ">>>=" => ReservedWord::UnsignedRightShiftAssign,
                    "&=" => ReservedWord::AndAssign,
                    "^=" => ReservedWord::XorAssign,
                    "|=" => ReservedWord::OrAssign,
                    "**=" => ReservedWord::ExponentialAssign,
                    _ => panic!(),
                };
                // self.tokens.push(TokenBase::Reserved(word));

                self.push_token(TokenBase::Reserved(word));

                replace_length = mat.len();
                self.col += replace_length as u32;
            } else if let Some(ident) = identifier_regex.find(&code) {
                let mat = ident.as_str();
                // self.tokens.push(TokenBase::Identifier(mat.to_string()));
                self.push_token(TokenBase::Identifier(mat.to_string()));

                replace_length = mat.len();
                self.col += replace_length as u32;
            } else if let Some(dq_str) = dq_str_literal_regex.find(&code) {
                let mat = dq_str.as_str().trim_matches('"');
                // self.tokens.push(TokenBase::String(mat.to_string()));
                self.push_token(TokenBase::String(mat.to_string()));

                replace_length = mat.len() + 2;
                self.col += replace_length as u32;
            } else if let Some(sq_str) = sq_str_literal_regex.find(&code) {
                let mat = sq_str.as_str().trim_matches('\'');
                // self.tokens.push(TokenBase::String(mat.to_string()));
                self.push_token(TokenBase::String(mat.to_string()));
                replace_length = mat.len() + 2;
                self.col += replace_length as u32;
            } else {
                let next = code.find(&['\r', '\n', '\t', ' ', '\0'][..]);
                match next {
                    Some(ind) if ind > 0 => {
                        invalid_tokens.push(self.push_invalid_token(code[..ind].to_string()));
                        replace_length = ind;
                    }
                    None => {
                        invalid_tokens.push(self.push_invalid_token(code[..].to_string()));
                        replace_length = code.len();
                    }
                    Some(_) => panic!("out of bound!"),
                }
            }
            self.ind += replace_length as u64;
            code.replace_range(0..replace_length, "");
        }
        if invalid_tokens.is_empty() {
            Ok(())
        } else {
            Err(LexError::new(invalid_tokens))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    /// expected: token_base, actually: token
    fn assert_eq_token_and_token_base(token_base: TokenBase, token: &Token) {
        assert_eq!(token.get_token().as_ref().unwrap(), &token_base);
    }

    #[test]
    fn test_lexer_single_comment() {
        let cases = vec![
            "// test1",
            "//test3\n",
            "//test4\r\n",
            "\n//ttest\n//te",
            "\n//\r\n",
        ];
        for item in cases {
            let mut lexer = Lexer::new(item, "");
            lexer.lex().unwrap();
            assert_eq!(0, lexer.tokens.len());
        }
    }
    #[test]
    fn test_identifier() {
        let cases = vec!["ident", "ident ident", "ide\nnt", "ode \t den"];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, "");
            lexer.lex().unwrap();

            if ind == 0 {
                assert_eq!(1, lexer.tokens.len());
            } else {
                let expected_idents = item
                    .split(|x| x == ' ' || x == '\n' || x == '\t')
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>();
                assert_eq!(lexer.tokens.len(), 2);
                assert_eq_token_and_token_base(
                    TokenBase::Identifier(expected_idents[0].to_string()),
                    &lexer.tokens[0],
                );
                assert_eq_token_and_token_base(
                    TokenBase::Identifier(expected_idents[1].to_string()),
                    &lexer.tokens[1],
                );
            }
        }
    }

    #[test]
    fn test_sq_string() {
        let cases = vec!["'test1'", "'test3''yrdy'", "'tes\nyr'", "'test\"te'"];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, "");
            if lexer.lex().is_ok() {
                match ind {
                    0 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test1")),
                            &lexer.tokens[0],
                        );
                    }
                    1 => {
                        assert_eq!(2, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test3")),
                            &lexer.tokens[0],
                        );
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("yrdy")),
                            &lexer.tokens[1],
                        );
                    }
                    3 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test\"te")),
                            &lexer.tokens[0],
                        );
                    }
                    _ => panic!(),
                }
            } else {
                assert_eq!(2, ind);
            }
        }
    }

    #[test]
    fn test_dq_string() {
        let cases = vec![
            r#""test1""#,
            r#""test3""yrdy""#,
            "\"tes\nyr\"",
            r#""test'te""#,
        ];
        for (ind, item) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, "");
            let result = lexer.lex();
            if result.is_ok() {
                match ind {
                    0 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test1")),
                            &lexer.tokens[0],
                        );
                    }
                    1 => {
                        assert_eq!(2, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test3")),
                            &lexer.tokens[0],
                        );
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("yrdy")),
                            &lexer.tokens[1],
                        );
                    }
                    3 => {
                        assert_eq!(1, lexer.tokens.len());
                        assert_eq_token_and_token_base(
                            TokenBase::String(String::from("test'te")),
                            &lexer.tokens[0],
                        );
                    }
                    _ => panic!(),
                }
            } else {
                assert_eq!(2, ind);
            }
        }
    }
    #[test]
    fn test_number() {
        let valid_cases = vec!["33", ".435", "3232.042", "0", "0.33"];
        for (ind, item) in valid_cases.iter().enumerate() {
            let mut lexer = Lexer::new(item, "");
            lexer.lex().unwrap();
            match ind {
                0 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq_token_and_token_base(
                        TokenBase::Number("33".to_string()),
                        &lexer.tokens[0],
                    );
                }
                1 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq_token_and_token_base(
                        TokenBase::Number(".435".to_string()),
                        &lexer.tokens[0],
                    );
                }
                2 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq_token_and_token_base(
                        TokenBase::Number("3232.042".to_string()),
                        &lexer.tokens[0],
                    );
                }
                3 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq_token_and_token_base(
                        TokenBase::Number("0".to_string()),
                        &lexer.tokens[0],
                    );
                }
                4 => {
                    assert_eq!(1, lexer.tokens.len());
                    assert_eq_token_and_token_base(
                        TokenBase::Number("0.33".to_string()),
                        &lexer.tokens[0],
                    );
                }
                _ => panic!(),
            }
        }
    }
    #[test]
    fn test_reserved() {
        let cases = vec![
            "=", "(", ")", "{", "}", "[", "]", ".", ",", ";", "=>", "const", "let", "import",
            "export", "default", "from", "true", "false", "+", "*", "/", "-", "%", "<", ">", "&",
            "|", "^", "~", "!", "<<", ">>", ">>>", "<=", ">=", "==", "!=", "**", "&&", "||", "+=",
            "-=", "*=", "/=", "%=", "<<=", ">>=", ">>>=", "&=", "^=", "|=", "**=", ":",
        ];
        use super::ReservedWord::*;
        use super::TokenBase::Reserved;
        for (ind, case) in cases.iter().enumerate() {
            let mut lexer = Lexer::new(case, "");
            lexer.lex().unwrap();

            assert_eq!(1, lexer.tokens.len());

            let first = lexer.tokens[0].get_token().as_ref().unwrap().clone();
            match ind {
                0 => assert_eq!(Reserved(Assign), first),
                1 => assert_eq!(Reserved(LeftParenthesis), first),
                2 => assert_eq!(Reserved(RightParenthesis), first),
                3 => assert_eq!(Reserved(LeftCurly), first),
                4 => assert_eq!(Reserved(RightCurly), first),
                5 => assert_eq!(Reserved(LeftSquareBracket), first),
                6 => assert_eq!(Reserved(RightSquareBracket), first),
                7 => assert_eq!(Reserved(Dot), first),
                8 => assert_eq!(Reserved(Comma), first),
                9 => assert_eq!(Reserved(SemiColon), first),
                10 => assert_eq!(Reserved(Arrow), first),
                11 => assert_eq!(Reserved(Const), first),
                12 => assert_eq!(Reserved(Let), first),
                13 => assert_eq!(Reserved(Import), first),
                14 => assert_eq!(Reserved(Export), first),
                15 => assert_eq!(Reserved(Default), first),
                16 => assert_eq!(Reserved(From), first),
                17 => assert_eq!(Reserved(True), first),
                18 => assert_eq!(Reserved(False), first),
                19 => assert_eq!(Reserved(Add), first),
                20 => assert_eq!(Reserved(Mult), first),
                21 => assert_eq!(Reserved(Div), first),
                22 => assert_eq!(Reserved(Sub), first),
                23 => assert_eq!(Reserved(Mod), first),
                24 => assert_eq!(Reserved(Less), first),
                25 => assert_eq!(Reserved(Greater), first),
                26 => assert_eq!(Reserved(And), first),
                27 => assert_eq!(Reserved(Or), first),
                28 => assert_eq!(Reserved(Xor), first),
                29 => assert_eq!(Reserved(Not), first),
                30 => assert_eq!(Reserved(LogicalNot), first),
                31 => assert_eq!(Reserved(LeftShift), first),
                32 => assert_eq!(Reserved(RightShift), first),
                33 => assert_eq!(Reserved(UnsignedRightShift), first),
                34 => assert_eq!(Reserved(LessOrEq), first),
                35 => assert_eq!(Reserved(GreaterOrEq), first),
                36 => assert_eq!(Reserved(Equal), first),
                37 => assert_eq!(Reserved(NotEqual), first),
                38 => assert_eq!(Reserved(Exponential), first),
                39 => assert_eq!(Reserved(LogicalAnd), first),
                40 => assert_eq!(Reserved(LogicalOr), first),
                41 => assert_eq!(Reserved(AdditiveAssign), first),
                42 => assert_eq!(Reserved(SubtractiveAssign), first),
                43 => assert_eq!(Reserved(MultiplicativeAssign), first),
                44 => assert_eq!(Reserved(DivisiveAssign), first),
                45 => assert_eq!(Reserved(ModuloAssign), first),
                46 => assert_eq!(Reserved(LeftShiftAssign), first),
                47 => assert_eq!(Reserved(RightShiftAssign), first),
                48 => assert_eq!(Reserved(UnsignedRightShiftAssign), first),
                49 => assert_eq!(Reserved(AndAssign), first),
                50 => assert_eq!(Reserved(XorAssign), first),
                51 => assert_eq!(Reserved(OrAssign), first),
                52 => assert_eq!(Reserved(ExponentialAssign), first),
                53 => assert_eq!(Reserved(Colon), first),
                // 54 => assert_eq!(Reserved(LeftShiftAssign), first),
                _ => panic!(),
            }
        }
    }
}

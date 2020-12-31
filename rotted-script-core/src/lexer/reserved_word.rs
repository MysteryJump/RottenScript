#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum ReservedWord {
    Equal = '=' as isize,
    LeftParenthesis = '(' as isize,
    RightParenthesis = ')' as isize,
    LeftCurly = '{' as isize,
    RightCurly = '}' as isize,
    LeftSquareBracket = '[' as isize,
    RightSquareBracket = ']' as isize,
    Period = '.' as isize,
    Comma = ',' as isize,
    SemiColon = ';' as isize,
    Arrow = 1000,
    Const,
    Let,
}

impl ToString for ReservedWord {
    fn to_string(&self) -> String {
        if *self < ReservedWord::Arrow {
            (*self as u8 as char).to_string()
        } else {
            String::from(match *self {
                ReservedWord::Arrow => "=>",
                ReservedWord::Const => "const",
                ReservedWord::Let => "let",
                _ => panic!(),
            })
        }
    }
}

use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum ReservedWord {
    Assign = '=' as isize,
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
    Import,
    Export,
    Default,
    From,
}

impl Display for ReservedWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if *self < ReservedWord::Arrow {
            (*self as u8 as char).to_string()
        } else {
            String::from(match *self {
                ReservedWord::Arrow => "=>",
                ReservedWord::Const => "const",
                ReservedWord::Let => "let",
                ReservedWord::Import => "import",
                ReservedWord::Export => "export",
                ReservedWord::Default => "default",
                ReservedWord::From => "from",
                _ => panic!(),
            })
        };
        write!(f, "{}", text)
    }
}

#[cfg(test)]
mod tests {
    use super::ReservedWord::*;

    #[test]
    fn test_to_string() {
        let reserveds = vec![
            Assign,
            LeftParenthesis,
            RightParenthesis,
            LeftCurly,
            RightCurly,
            LeftSquareBracket,
            RightSquareBracket,
            Period,
            Comma,
            SemiColon,
            Arrow,
            Const,
            Let,
            Import,
            Export,
            Default,
            From,
        ];
        for item in reserveds {
            match item {
                Assign => assert_eq!("=", item.to_string()),
                LeftParenthesis => assert_eq!("(", item.to_string()),
                RightParenthesis => assert_eq!(")", item.to_string()),
                LeftCurly => assert_eq!("{", item.to_string()),
                RightCurly => assert_eq!("}", item.to_string()),
                LeftSquareBracket => assert_eq!("[", item.to_string()),
                RightSquareBracket => assert_eq!("]", item.to_string()),
                Period => assert_eq!(".", item.to_string()),
                Comma => assert_eq!(",", item.to_string()),
                SemiColon => assert_eq!(";", item.to_string()),
                Arrow => assert_eq!("=>", item.to_string()),
                Const => assert_eq!("const", item.to_string()),
                Let => assert_eq!("let", item.to_string()),
                Import => assert_eq!("import", item.to_string()),
                Export => assert_eq!("export", item.to_string()),
                Default => assert_eq!("default", item.to_string()),
                From => assert_eq!("from", item.to_string()),
            }
        }
    }
}
